// #[clippy::deny()]

use std::io::Write;
use std::path::PathBuf;

#[macro_use]
extern crate test_bin;

mod clang_singleton;
use clang_singleton::check_c_file_parses;

const DEFAULT_WARNING_FLAGS: &[&str] = &["-Wall", "-Wextra", "-Wpedantic"];
const SMOKE_TEST_FILE: &str = "tests/smoke_test.h";

/// Verify that diagnostics actually cause the test to fail
#[test]
#[should_panic]
fn verify_diagnostics_fail_test() {
    // where to generate to
    let mut header_file = tempfile::Builder::new().suffix(".h").tempfile().unwrap();

    write!(header_file, "void a(){{}}").unwrap();

    // check the results
    check_c_file_parses(header_file.path(), DEFAULT_WARNING_FLAGS).unwrap();
}

macro_rules! gen_test {
    ( $( $arg:literal $( = [ $( $value:literal )+ ] )? ),* ) => {
        paste::paste! {
            #[test]
            fn [< generate $( _ $arg $( $( _ $value )+ )? )* >]() {
                // file to read in
                let input_file = PathBuf::from(SMOKE_TEST_FILE);

                // where to generate to
                let prototype_file = tempfile::Builder::new()
                    .prefix(input_file.file_stem().unwrap())
                    .suffix(".h")
                    .tempfile()
                    .unwrap();

                // debug info
                eprintln!("input file: {input_file:?}\noutput file: {prototype_file:?}");

                let args: &[String] = &[
                    $( String::from("--") + $arg $(, [ $( String::from($value) ),+ ].join(" ") )? ),*
                ];

                // actual processing
                let _output = get_test_bin!("c-abi-lens")
                    .args(args)
                    .arg("--output-file")
                    .arg(prototype_file.path().as_os_str())
                    .arg(input_file)
                    .output()
                    .unwrap();

                // check the results
                check_c_file_parses(prototype_file.path(), DEFAULT_WARNING_FLAGS).unwrap();
            }
        }
    };
}

gen_test! {}
gen_test! {"comment"}
gen_test! {"comment", "only-prototype"}
gen_test! {"prefix" = ["funky_prefix"], "comment", "only-prototype"}
gen_test! {"function-decl-prefix" = [ "inline" ], "comment"}
gen_test! {"function-decl-prefix" = [ "static" "inline" ], "comment"}
gen_test! {"function-decl-prefix" = [ "static" "inline" ], "comment", "endianness-swap"}
