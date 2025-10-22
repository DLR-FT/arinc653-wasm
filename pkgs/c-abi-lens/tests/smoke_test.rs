// #[clippy::deny()]

use std::io::Write;
use std::path::PathBuf;

#[macro_use]
extern crate test_bin;

const DEFAULT_WARNING_FLAGS: &[&str] = &["-Wall", "-Wextra", "-Wpedantic"];
const SMOKE_TEST_FILE: &str = "tests/smoke_test.h";

/// Compile the `input_file` with a given set of `clang_args`
///
/// Panics if compilation fails or causes any kind of diagnostics
///
/// The final arguments passed to clang are from environment variables suffixed by `clang_args`. The
/// reason for this choice is: Environment variables allow the user to configure sysroot etc. for
/// all tests, so that clang can actually compile things. The `clang_args` allow the testsuite's
/// individual tests to modify the compilation as they see fit.
fn check_c_file_parses<F: Into<PathBuf>>(input_file: F, clang_args: &[&str]) {
    // clang ini boilerplate
    let clang = clang::Clang::new().unwrap();
    let index = clang::Index::new(&clang, false, true);
    let input_path_buf = input_file.into();

    // check file exists, clang itself does not provide a useful error message if the input file
    // does not exists
    if !std::fs::exists(&input_path_buf).unwrap() {
        panic!("file {input_path_buf:?}");
    }

    // prepare the final clang_arguments, by extend the environment variable based ones with the
    // `clang_args` passed to this function
    let mut default_clang_args = c_abi_lens::clang_args_from_env();
    default_clang_args.extend(clang_args.iter().map(|s| s.to_string()));
    let clang_args = default_clang_args;
    eprintln!("clang args: {clang_args:#?}");

    // parse and translate the file
    let mut parser = index.parser(input_path_buf);
    parser.arguments(&clang_args).keep_going(true);
    let tu = parser.parse().unwrap();

    // collect all diagnostics
    let diagnostics = tu
        .get_diagnostics()
        .into_iter()
        .map(|d| d.formatter().format())
        .fold(String::new(), |mut acc, d| {
            acc.push('\n');
            acc.push_str(&d);
            acc
        });

    // fail if there are any diagnostics
    if !diagnostics.is_empty() {
        panic!("{diagnostics}")
    }
}

/// Verify that diagnostics actually cause the test to fail
#[test]
#[should_panic]
fn verify_diagnostics_fail_test() {
    // where to generate to
    let mut header_file = tempfile::Builder::new().suffix(".h").tempfile().unwrap();

    write!(header_file, "void a(){{}}").unwrap();

    // check the results
    check_c_file_parses(header_file.path(), DEFAULT_WARNING_FLAGS);
}

#[test]
fn generate_prototype_native() {
    // file to read in
    let input_file = PathBuf::from(SMOKE_TEST_FILE);

    // where to generate to
    let prototype_file = tempfile::Builder::new()
        .prefix(input_file.file_stem().unwrap())
        .suffix(".h")
        .tempfile()
        .unwrap();

    // debug info
    eprintln!("input file: {input_file:?}\nprototype file: {prototype_file:?}");

    // actual processing
    let _output = get_test_bin!("c-abi-lens")
        .args(["--comment", "--only-prototype", "--output-file"])
        .arg(prototype_file.path().as_os_str())
        .arg(input_file)
        .output()
        .unwrap();

    // check the results
    check_c_file_parses(prototype_file.path(), DEFAULT_WARNING_FLAGS);
}
