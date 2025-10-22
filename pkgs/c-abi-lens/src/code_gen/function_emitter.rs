use color_eyre::{
    Result, Section,
    eyre::{OptionExt, bail, ensure},
};
use log::{debug, error, info};

use crate::code_gen::RepresentableCType;

use super::{CFunc, CSection, CSnippet};

/// Emit all functions for a given struct
pub fn insert_struct_functions(
    code_snippets: &mut Vec<CSnippet>,
    struct_: &clang::Entity,
    swap_endianness: bool,
) -> Result<()> {
    let struct_type = struct_.get_type().ok_or_eyre("struct type is unknown?!")?;
    let struct_size_bytes = struct_type.get_sizeof()?;

    let struct_name = struct_.get_name().ok_or_eyre("struct has no name")?;
    info!("generating for struct {struct_name:?}");

    debug!("struct: {struct_name:?} (size: {struct_size_bytes} bytes)");

    // per-struct functions
    emit_per_struct_functions(code_snippets, &struct_name, struct_type)?;
    code_snippets.push(CSnippet::Newline);

    // per-struct-field functions
    for struct_field in struct_.get_children() {
        ensure!(
            struct_field.get_kind() == clang::EntityKind::FieldDecl,
            "all fields of a struct must be of FieldDecl type"
        );

        // note down the origin of this error
        let error_origin = format!("struct {struct_name:?}, field {struct_field:?}");

        let field_name = struct_field
            .get_name()
            .ok_or_eyre("unknown name")
            .section(error_origin.clone())?;

        // get offset of the field and its type, fail gracefully (by just ignoring the field)
        let (field_offset_bits, field_ty) = match (
            struct_type.get_offsetof(&field_name),
            struct_field.get_type(),
        ) {
            (Ok(fo), Some(ft)) => (fo, ft),
            (Ok(_), None) => {
                error!("skipping {error_origin}: it has no known field type");
                continue;
            }
            (Err(e), Some(_)) => {
                error!("skipping {error_origin}: getting its offsetof yielded an error:\n{e}");
                continue;
            }
            (Err(e), None) => {
                error!(
                    "skipping {error_origin}: it has no known field type and getting its offsetof yielded an error:\n{e}"
                );
                continue;
            }
        };

        debug!(
            "    field: {field_name:?} (offset: {} bits)",
            field_offset_bits
        );

        if let Err(e) = emit_per_field_functions(
            code_snippets,
            &struct_name,
            &field_name,
            field_offset_bits,
            field_ty,
            swap_endianness,
        ) {
            error!(
                "generating the per-field functions for {error_origin} yielded the following error, skipping it\n{e}"
            );
        };
    }
    code_snippets.push(CSnippet::Newline);

    Ok(())
}

/// Insert the [`CSnippets`] for functions related to a given struct itself
fn emit_per_struct_functions(
    code_snippets: &mut Vec<CSnippet>,
    struct_name: &str,
    struct_type: clang::Type,
) -> Result<()> {
    let namespace_prefix = "cal";
    let function_name_gen = |op| format!("{namespace_prefix}_{op}__{struct_name}");

    let struct_size_bytes = struct_type.get_sizeof()?;

    // section header for this struct
    code_snippets.push(
        CSection {
            title: format!(" {struct_name} "),
            comment: Default::default(),
        }
        .into(),
    );
    code_snippets.push(CSnippet::Newline);

    // helper functions for size of the entire struct
    code_snippets.push(CSnippet::from(CFunc {
        comment: format!(
            "\
                `sizeof({struct_name})`\n\
                \n\
                Returns the size in bytes consumed for one instance of the `{struct_name}`\
            "
        ),
        return_type: RepresentableCType::UIntPtr,
        name: function_name_gen("sizeof"),
        arguments: vec![],
        body: format!("return {};", struct_size_bytes),
    }));
    code_snippets.push(CSnippet::Newline);

    Ok(())
}

/// Insert the [`CSnippets`] for functions related to a given struct's field
fn emit_per_field_functions(
    code_snippets: &mut Vec<CSnippet>,
    struct_name: &str,
    field_name: &str,
    offset_bits: usize,
    ty: clang::Type,
    swap_endianness: bool,
) -> Result<()> {
    if offset_bits % 8 != 0 {
        bail!("bit offset which is not devisable by 8, this is not implemented yet");
    }

    let offset_bytes = offset_bits / 8;

    let namespace_prefix = "cal";
    let function_name_gen = |op| format!("{namespace_prefix}_{op}__{struct_name}__{field_name}");

    // desugar this type so that we know what it actually is
    let canonical_type = ty.get_canonical_type();

    // a couple of often reccuring C types
    let c_void_ptr_ty = RepresentableCType::Opaque { bytes: None };
    let c_u8_ty = RepresentableCType::Integer {
        bytes: 1,
        is_unsigned: true,
    };

    // find a platform agnostic representation of this type
    let generic_c_field_repr = RepresentableCType::new(&canonical_type)?;

    code_snippets.push(
        CSection {
            title: format!(" {struct_name}.{field_name} "),
            comment: Default::default(),
        }
        .into(),
    );
    code_snippets.push(CSnippet::Newline);

    // helper functions for size of the field
    code_snippets.push(
        CFunc {
            comment: format!("\
                `sizeof({struct_name}->{field_name})`\n\
                \n\
                Returns the size in bytes of the `{field_name}` field from the `{struct_name}` struct\
            "),
            return_type: RepresentableCType::UIntPtr,
            name: function_name_gen("sizeof"),
            arguments: vec![],
            body: format!("return {};", generic_c_field_repr.total_size_bytes()?),
        }
        .into(),
    );
    code_snippets.push(CSnippet::Newline);

    // helper functions for offset of the field withing the struct
    code_snippets.push(CFunc {
        comment: format!("\
            `offsetof({struct_name}, {field_name})`\n\
            \n\
             Get the offset in bytes of the `{field_name}` field from the start of a `{struct_name}` struct\
        "),
        return_type: RepresentableCType::UIntPtr,
        name: function_name_gen("offsetof"),
        arguments: vec![],
        body: format!("return {offset_bytes};"),
    }.into());
    code_snippets.push(CSnippet::Newline);

    // string to anounce the presence of byte-swapping
    let maybe_endianness_swapped = if swap_endianness {
        ", with endianness swapped"
    } else {
        ""
    };

    use clang::TypeKind::*;
    match (canonical_type.get_kind(), canonical_type.get_element_type()) {
        // integer or float or pointer
        (
            CharS | CharU | SChar | UChar | Short | UShort | Int | UInt | Long | ULong | LongLong
            | ULongLong | Float | Double | Enum,
            _,
        ) => {
            let size_of_type_bytes = generic_c_field_repr.total_size_bytes()?;
            let byte_swap_fn = match size_of_type_bytes {
                1 => "".to_owned(),
                n @ 2 | n @ 4 | n @ 8 => format!("bswap_{}", 8 * n),
                n => {
                    bail!("unable to perform a byte swap for an integer that is {n} bytes wide")
                }
            };

            // getter for integer types
            code_snippets.insert(code_snippets.len() -2, CFunc {
                comment: format!("\
                    Get `{struct_name}.{field_name}`\n\
                    \n\
                    Returns the field `{field_name}`'s value from an instance of the `{struct_name}` struct{maybe_endianness_swapped}\
                "),
                return_type: generic_c_field_repr.clone(),
                name: function_name_gen("get"),
                arguments: [
                    (c_void_ptr_ty.clone(), "struct_base_addr".to_owned())
                ].into(),
                body: if swap_endianness {
                    format!("return {byte_swap_fn}(*({generic_c_field_repr}*)(({generic_c_field_repr}*) struct_base_addr + {offset_bytes}));")
                }else {
                    format!("return *({generic_c_field_repr}*)(({generic_c_field_repr}*) struct_base_addr + {offset_bytes});")
                }
            }.into());
            code_snippets.insert(code_snippets.len() - 2, CSnippet::Newline);

            // setter for integer types
            code_snippets.insert(code_snippets.len() -2, CFunc {
                comment: format!("\
                    Set `{struct_name}.{field_name}` to `value`\n\
                    \n\
                    Overwrites the field `{field_name}`'s value of an `{struct_name}` struct instance with `value`{maybe_endianness_swapped}\
                "),
                return_type: RepresentableCType::Void,
                name: function_name_gen("set"),
                arguments: [
                    (c_void_ptr_ty.clone(), "struct_base_addr".to_owned()),
                    (generic_c_field_repr.clone(), "value".to_owned())
                ].into(),
                body: if swap_endianness {
                    format!("*(({generic_c_field_repr}*) struct_base_addr + {offset_bytes}) = {byte_swap_fn}(value);")
                }else {
                    format!("*(({generic_c_field_repr}*) struct_base_addr + {offset_bytes}) = value;")
                }
            }.into());
            code_snippets.insert(code_snippets.len() - 2, CSnippet::Newline);
        }

        // for arrays whose element type is one byte in size we can just verbatim copy the elements
        // TODO be more elegant, add more representations than 1D of u8
        (ConstantArray, Some(_)) if generic_c_field_repr.element_size_bytes()? == 1 => {
            let total_bytes = generic_c_field_repr.total_size_bytes()?;

            let return_type = RepresentableCType::Array {
                element_type: Box::new(c_u8_ty),
                length: total_bytes,
            };
            let argument_type = return_type.clone();

            // getter for array types
            code_snippets.insert(code_snippets.len() -2, CFunc {
                comment: format!("\
                    Read from `{struct_name}.{field_name}`\n\
                    \n\
                    Copies from `{field_name}` field of an instance of the `{struct_name}` struct to `destination`\
                "),
                return_type: RepresentableCType::Void,
                name: function_name_gen("read"),
                arguments: [
                    (c_void_ptr_ty.clone(), "struct_base_addr".to_owned()),
                    (argument_type.clone(), "dst".to_owned())
                ].into(),
                body: format!("\
                    for(uintptr_t i = 0; i < {total_bytes}; i++)\n\
                    \tdst[i] = ((uint8_t*)struct_base_addr)[{offset_bytes} + i];\
                ")
            }.into());
            code_snippets.insert(code_snippets.len() - 2, CSnippet::Newline);

            // setter for array types
            code_snippets.insert(code_snippets.len() -2, CFunc {
                comment: format!("\
                    Write to `{struct_name}.{field_name}`\n\
                    \n\
                    Copies from `source` to the `{field_name}` field of an `{struct_name}` struct instance\
                "),
                return_type: RepresentableCType::Void,
                name: function_name_gen("write"),
                arguments: [
                    (c_void_ptr_ty.clone(), "struct_base_addr".to_owned()),
                    (argument_type.clone(), "src".to_owned())
                ].into(),
                body: format!("\
                    for(uintptr_t i = 0; i < {total_bytes}; i++)\n\
                    \t((uint8_t*)struct_base_addr)[{offset_bytes} + i] = src[i];\
                ")
            }.into());
            code_snippets.insert(code_snippets.len() - 2, CSnippet::Newline);
        }

        // we don't know what to do, so just hand out a void pointer
        (type_kind, maybe_type) => {
            // accessor via void ptr
            code_snippets.insert(code_snippets.len() -2, CFunc {
                comment: format!("\
                    Get void pointer to `{struct_name}.{field_name}`\n\
                    \n\
                    No ABI compatible representation of this type is known, therefore this just returns a void ptr\n\
                    \n\
                    type kind:  {type_kind:?}\n\
                    maybe type: {maybe_type:?}\
                "),
                return_type: c_void_ptr_ty.clone(),
                name: function_name_gen("get"),
                arguments: [
(                    c_void_ptr_ty.clone(), "struct_base_addr".to_owned()),
                ].into(),
                body: format!("return (void*)((uint8_t*)struct_base_addr + {offset_bytes});")
            }.into());
            code_snippets.insert(code_snippets.len() - 2, CSnippet::Newline);
        }
    }

    Ok(())
}
