use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Whether to perform an endianness swap
    ///
    /// If this flag is not set, endianness is never changed.
    /// If this flag is set, endianness of all primitive types longer than 1 byte is swapped.
    #[arg(short, long)]
    pub endianness_swap: bool,

    /// Input C file to consume
    ///
    /// Can be either a .c or a .h file.
    pub input_file: PathBuf,

    /// Output C file
    ///
    /// If not specified, instead the generated code is printed to the terminal's stdout
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,

    /// Identifier prefix
    ///
    /// Prefix used before all visible identifiers
    #[arg(short, long, default_value = "cal")]
    pub prefix: String,

    /// Function declaration prefix
    ///
    /// Set a prefix for function declarition, e.g. `static inline`
    #[arg(short, long)]
    pub function_decl_prefix: Option<String>,

    /// Emit comment for each function
    ///
    /// If set, each function comes with a doc-comment
    #[arg(short, long)]
    pub comment: bool,

    /// Only emit prototype/forward declaration for each function
    ///
    /// If set, each function will only be declared but not defined
    #[arg(long)]
    pub only_prototype: bool,

    /// Clang arguments
    ///
    /// These are passed through verbatim to (lib-)clang. Likely you want to set the target
    /// architecture here, e.g.
    ///
    /// -- --target=wasm32-unknown-none
    ///
    /// These are prefixed with the space separated contents of either the EXTRA_CLANG_ARGS and/or
    /// the BINDGEN_EXTRA_CLANG_ARGS environment variables. The intent behind this mechanism is, to
    /// allow setting up sysroot and other essentials for libclang via environment variables.
    ///
    /// In particular, the choice of BINDGEN_EXTRA_CLANG_ARGS makes it so that a Nix build
    /// environment with the `rustPlatform.bindgenHook` in `nativeBuildInputs` will just work.
    pub clang_args: Vec<String>,
}
