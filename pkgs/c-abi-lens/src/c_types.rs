use clang::TypeKind;
use color_eyre::{
    Result, Section,
    eyre::{ensure, eyre},
};
use log::error;

/// Trait for types that can be formatted to C code
pub trait FormatToCType {
    /// Generate the string representing this type in C for a function return value
    fn to_function_return_type(&self) -> String;

    /// Generate the string representing this type in C for a function argument
    fn to_function_argument_type(&self, argument_name: &str) -> String;

    /// Get the size in bytes of this type as it would be return by `sizeof()` on the target architecture
    fn size_bytes(&self) -> usize;
}

/// Represents integer types in C using `stdint.h` based types
pub struct CIntegerStdintH {
    bytes: usize,
    is_unsigned: bool,
}

impl CIntegerStdintH {
    pub fn new(ty: clang::Type) -> Result<Self> {
        ensure!(
            ty.is_integer() || ty.get_pointee_type().is_some() || ty.get_kind() == TypeKind::Enum,
            eyre!("translate_integer_type can only translate integer, pointer and enum types")
                .with_warning(|| format!("type in question: {ty:?}"))
        );

        let value_size_bytes = ty
            .get_sizeof()
            .with_note(|| format!("type {ty:?} does not have a size"))?;

        let valid_sizes = [1, 2, 4, 8];
        ensure!(
            valid_sizes.contains(&value_size_bytes),
            eyre!(
                "cannot create CIntegerStdintH from type {ty:?}, it's size is {value_size_bytes} bytes but only {valid_sizes:?} are supported"
            )
        );

        Ok(Self {
            bytes: value_size_bytes,
            is_unsigned: ty.is_unsigned_integer(),
        })
    }
}

impl FormatToCType for CIntegerStdintH {
    fn to_function_return_type(&self) -> String {
        let Self { bytes, is_unsigned } = self;
        let bits = bytes * 8;
        format!("{}int{bits}_t", if *is_unsigned { "u" } else { "" })
    }

    fn to_function_argument_type(&self, argument_name: &str) -> String {
        let argument_type = Self::to_function_return_type(self);
        format!("{argument_type} {argument_name}")
    }

    fn size_bytes(&self) -> usize {
        self.bytes
    }
}

// Represent an array of (un-)signed bytes in C using `stdint.h`'s `uint8_t` type
pub struct CByteArrayType {
    bytes: usize,
}

impl CByteArrayType {
    pub fn new(ty: clang::Type) -> Result<Self> {
        // ensure!(
        //     ty.is_integer(),
        //     eyre!("translate_integer_type can only translate integer types")
        //         .with_warning(|| format!("type in question: {ty:?}"))
        // );

        let value_size_bytes = ty
            .get_sizeof()
            .with_note(|| format!("type {ty:?} does not have a size"))?;

        Ok(Self {
            bytes: value_size_bytes,
        })
    }
}

impl FormatToCType for CByteArrayType {
    fn to_function_return_type(&self) -> String {
        "uint8_t".to_string()
    }

    fn to_function_argument_type(&self, argument_name: &str) -> String {
        let Self { bytes, .. } = self;
        let argument_type = Self::to_function_return_type(self);
        format!("{argument_type} {argument_name}[{bytes}]")
    }

    fn size_bytes(&self) -> usize {
        self.bytes
    }
}


// Represent an unrepresentable type
#[derive(Debug)]
pub struct UnrepresentableType<'a>{
    pub type_kind:  clang::TypeKind,
    pub maybe_type:  Option<clang::Type<'a>>
}

impl<'a> FormatToCType for UnrepresentableType<'a> {

    fn to_function_return_type(&self) -> String {
        "void*".to_string()
    }

    fn to_function_argument_type(&self, argument_name: &str) -> String {
         format!( "{} {argument_name}", Self::to_function_return_type(self))
    }

    fn size_bytes(&self) -> usize {
        let Self { type_kind, maybe_type } = self;

        match maybe_type.map(|t| t.get_sizeof()) {
            Some(Ok(size)) => size,
            Some(Err(e)) => {error!("Got error while figuring out sizeof {type_kind:?}, returning 0 instead:\n{e}"); 0},
            None => {error!("No size information available for {type_kind:?}, returning 0 instead"); 0},
        }
    }
}
