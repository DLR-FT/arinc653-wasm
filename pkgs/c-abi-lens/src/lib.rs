use log::{debug, trace};

/// Get default clang args from environment variables
pub fn clang_args_from_env() -> Vec<String> {
    debug!("adding additional clang_args from environment variables");
    ["EXTRA_CLANG_ARGS", "BINDGEN_EXTRA_CLANG_ARGS"]
        .into_iter()
        .inspect(|s| trace!("reading {s} environment variable"))
        .map(std::env::var)
        .filter_map(Result::ok)
        .flat_map(|s| {
            s.split_whitespace()
                .map(String::from)
                .collect::<Vec<String>>()
        })
        .collect()
}
