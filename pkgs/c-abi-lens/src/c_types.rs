use clang::TypeKind;
use color_eyre::{Result, eyre::bail};

/// An enum containing all possible representations of C types known to this library
///
/// For sized types, bails out to at least providing an Opaque void pointer and a size.
/// For unsized types, does nothing.
#[derive(Debug, Clone)]
pub enum RepresentableCType {
    Integer {
        bytes: u8,
        is_unsigned: bool,
    },
    Float {
        bytes: u8,
    },
    Array {
        element_type: Box<RepresentableCType>,
        dimensional_lengths: Vec<u64>,
    },
    Opaque {
        bytes: Option<u64>,
    },
    UIntPtr,
    Void,
}

impl RepresentableCType {
    /// Create a new [`RepresentableCType`] instance, based on a [`clang::Type`]
    ///
    /// Tries to find a suitable, platform independent representation.
    pub fn new(type_: &clang::Type) -> Result<Self> {
        let kind = type_.get_kind();
        let size_of = type_.get_sizeof()?;
        let element_type = type_.get_element_type().map(|et| Self::new(&et));
        // let element_type_size_of = element_type.map(|et| et.get_sizeof()).transpose()?;

        use TypeKind::*;
        match (kind, size_of, element_type) {
            // its an int of some sorts
            (
                CharS | CharU | SChar | UChar | Short | UShort | Int | UInt | Long | ULong
                | LongLong | ULongLong,
                1 | 2 | 4 | 8,
                _,
            ) => {
                let is_unsigned = type_.is_unsigned_integer();
                let size = size_of.try_into().unwrap(); // 1 | 2 | 4 | 8 all fit into an u8
                Ok(RepresentableCType::Integer {
                    bytes: size,
                    is_unsigned,
                })
            }

            // its a float of some sorts
            (Float | Double, 4 | 8, _) => {
                let size = size_of.try_into().unwrap(); // 4 | 8 all fit into an u8
                Ok(RepresentableCType::Float { bytes: size })
            }

            (ConstantArray, _, Some(Ok(element_type_repr))) => {
                let length = type_
                    .get_size()
                    .unwrap()
                    .try_into()
                    .expect("unable to fit type size into an u64");
                // .ok_or(|| eyre!("array {type_:?} does not have a known length"))?;

                Ok(Self::Array {
                    element_type: Box::new(element_type_repr),
                    dimensional_lengths: vec![length], // TODO find out how to get multi dimensional lengths
                })
            }

            // we don't know what to do, so just hand out a void pointer
            (_type_kind, type_size, _) => Ok(RepresentableCType::Opaque {
                bytes: Some(
                    type_size
                        .try_into()
                        .expect("unable to fit type size into an u64"),
                ),
            }),
        }
    }

    /// Generate a C code snippt for this [`RepresentableCType`], optionally for a named variable/argument
    pub fn format_as_type(&self, var_name: Option<&str>) -> String {
        let maybe_var_name_with_space_prefix =
            var_name.map(|x| format!(" {x}")).unwrap_or_default();
        match self {
            Self::Integer {
                bytes,
                is_unsigned: true,
            } => {
                let bits = *bytes as u16 * 8;
                format!("uint{bits}_t{maybe_var_name_with_space_prefix}")
            }
            Self::Integer {
                bytes,
                is_unsigned: false,
            } => format!("int{bytes}_t{maybe_var_name_with_space_prefix}"),

            Self::Float { bytes: 4 } => {
                format!("float{maybe_var_name_with_space_prefix}")
            }
            Self::Float { bytes: 8 } => {
                format!("double{maybe_var_name_with_space_prefix}")
            }
            Self::Float { bytes } => panic!("unable to represent a {bytes} float"),
            Self::Array {
                element_type,
                dimensional_lengths,
            } if matches!(
                **element_type,
                Self::Integer { .. } | Self::Float { .. } | Self::Opaque { .. }
            ) =>
            {
                let mut base_type = element_type.format_as_type(None);
                base_type.push_str(&maybe_var_name_with_space_prefix);
                base_type.extend(dimensional_lengths.iter().map(|d| format!("[{d}]")));

                base_type
            }
            Self::Opaque { bytes: _ } => {
                format!("void*{maybe_var_name_with_space_prefix}")
            }
            Self::UIntPtr => "uintptr_t".into(),
            Self::Void => "void".into(),
            _ => {
                panic!(
                    "instance {self:#?} should not have been constructed; we don't know how to represent that in C"
                );
            }
        }
    }

    /// Get the size in bytes of this type (not including possible padding)
    ///
    /// For arrays, this returns the size of the entire array
    pub fn size_bytes(&self) -> Result<u64> {
        Ok(match self {
            Self::Array {
                element_type,
                ..
                // dimensional_lengths,
            } => {
                let bytes_per_element = &element_type.size_bytes()?;
                let total_elements: u64 = self.size_element_bytes()?;
                bytes_per_element * total_elements
            }
            _ => self.size_element_bytes()?
        })
    }

    /// Get the size in bytes of this type (not including possible padding)
    ///
    /// For arrays, this returns the size of an individual element
    pub fn size_element_bytes(&self) -> Result<u64> {
        Ok(match self {
            Self::Integer { bytes, .. } => (*bytes).into(),
            Self::Float { bytes } => (*bytes).into(),
            Self::Array { element_type, .. } => {
                assert!(
                    !matches!(&**element_type, Self::Array { .. },),
                    "nested arrays cause recursion for this function and shall never be constructed"
                );

                element_type.size_bytes()?
            }
            Self::Opaque { bytes: Some(bytes) } => *bytes,
            Self::Opaque { bytes: None } | Self::UIntPtr | Self::Void => {
                bail!("type {self:?} has no known size")
            }
        })
    }

    /// Get the length in elements, or `1` otherwise
    ///
    /// Returns the product of all dimensions if this is multi-dimensional
    pub fn length(&self) -> u64 {
        match self {
            Self::Integer { .. }
            | Self::Float { .. }
            | Self::Opaque { .. }
            | Self::UIntPtr
            | Self::Void => 1,
            Self::Array {
                dimensional_lengths,
                ..
            } => dimensional_lengths.iter().product(),
        }
    }
}

impl std::fmt::Display for RepresentableCType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format_as_type(None))
    }
}
