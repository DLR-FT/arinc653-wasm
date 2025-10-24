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
        length: u64,
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
                | LongLong | ULongLong | Enum,
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

            // a 1 or multidimensional array of one given type
            (ConstantArray, _, Some(Ok(element_type_repr))) => {
                let length = type_
                    .get_size()
                    .unwrap()
                    .try_into()
                    .expect("unable to fit type size into an u64");

                Ok(Self::Array {
                    element_type: Box::new(element_type_repr),
                    length,
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
            } => {
                let bits = *bytes as u16 * 8;
                format!("int{bits}_t{maybe_var_name_with_space_prefix}")
            }

            Self::Float { bytes: 4 } => {
                format!("float{maybe_var_name_with_space_prefix}")
            }
            Self::Float { bytes: 8 } => {
                format!("double{maybe_var_name_with_space_prefix}")
            }
            Self::Float { bytes } => panic!("unable to represent a {bytes} float"),
            Self::Array { .. } => {
                let mut base_type = self.element_type().format_as_type(None);
                base_type.push_str(&maybe_var_name_with_space_prefix);
                self.recurse_into_type(|c_type, _, is_last| {
                    if !is_last {
                        base_type.push_str(&format!("[{}]", c_type.length_1d()));
                    }
                });

                base_type
            }
            Self::Opaque { bytes: _ } => {
                format!("void *{maybe_var_name_with_space_prefix}")
            }
            Self::UIntPtr => "uintptr_t".into(),
            Self::Void => "void".into(),
        }
    }

    /// Get the size in bytes of this type
    ///
    /// For arrays, this returns the size of the entire array
    pub fn total_size_bytes(&self) -> Result<u64> {
        let element_size = self.element_size_bytes()?;
        let length = self.length();
        Ok(element_size * length)
    }

    /// Get the size in bytes of this type (not including possible padding)
    ///
    /// For arrays, this returns the size of an individual element
    pub fn element_size_bytes(&self) -> Result<u64> {
        Ok(match self {
            Self::Integer { bytes, .. } => (*bytes).into(),
            Self::Float { bytes } => (*bytes).into(),
            Self::Array { .. } => self.element_type().element_size_bytes()?,
            Self::Opaque { bytes: Some(bytes) } => *bytes,
            Self::Opaque { bytes: None } | Self::UIntPtr | Self::Void => {
                bail!("type {self:?} has no known size")
            }
        })
    }

    pub fn element_type(&self) -> RepresentableCType {
        match self {
            Self::Array { element_type, .. } => {
                let mut base_type = None;
                element_type.recurse_into_type(|c_type, _, is_last| {
                    if is_last {
                        base_type = Some(c_type.clone())
                    }
                });

                base_type.expect("recurse_into_type must always terminate with a last element true")
            }
            x => x.clone(),
        }
    }

    /// Get the length in elements of an array's first dimension, or `1` otherwise
    ///
    /// Returns **only** the first dimension length for arrays
    pub fn length_1d(&self) -> u64 {
        match self {
            Self::Integer { .. }
            | Self::Float { .. }
            | Self::Opaque { .. }
            | Self::UIntPtr
            | Self::Void => 1,
            Self::Array { length, .. } => *length,
        }
    }

    /// Get the length in elements of an array, or `1` otherwise
    ///
    /// Returns the **product** of all dimensions if this is multi-dimensional array
    pub fn length(&self) -> u64 {
        match self {
            Self::Integer { .. }
            | Self::Float { .. }
            | Self::Opaque { .. }
            | Self::UIntPtr
            | Self::Void => 1,
            Self::Array { .. } => {
                let mut length = 1;
                self.recurse_into_type(|c_type, _, _| length *= c_type.length_1d());
                length
            }
        }
    }

    /// Recurse into a nested type, calling a closure for each layer
    ///
    /// Implemented without function recursion.
    ///
    fn recurse_into_type<F: FnMut(&RepresentableCType, bool, bool)>(&self, mut f: F) {
        let mut c_type = self;
        let mut next_c_type = self;
        let mut is_first = true;
        let mut is_last;

        loop {
            match c_type {
                RepresentableCType::Array { element_type, .. } => {
                    // if the recursion continues, this is not the last element!
                    is_last = false;
                    // mark the next element
                    next_c_type = element_type;
                }
                _ => {
                    // if this is not a recursive variant, the recursion ends!
                    is_last = true;
                }
            }

            // run the actual closure
            f(c_type, is_first, is_last);

            if is_last {
                // if this was the last recursion, break
                break;
            }

            // else advance `c_type` and make sure this was the only time `is_first` is `true`
            c_type = next_c_type;
            is_first = false;
        }
    }
}

impl std::fmt::Display for RepresentableCType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format_as_type(None))
    }
}

#[cfg(test)]
mod test {
    use super::RepresentableCType;

    #[test]
    fn test_format_array() {
        let arr = RepresentableCType::Array {
            element_type: Box::new(RepresentableCType::Array {
                element_type: Box::new(RepresentableCType::Integer {
                    bytes: 4,
                    is_unsigned: true,
                }),
                length: 11,
            }),
            length: 7,
        };

        assert_eq!(arr.format_as_type(Some("arr")), "uint32_t arr[7][11]")
    }
}
