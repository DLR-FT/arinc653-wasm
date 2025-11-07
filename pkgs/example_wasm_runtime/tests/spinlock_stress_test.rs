use example_wasm_runtime::run;

#[test]
fn spinlock_stress_test() {
    let results = run(
        "host",
        "shared_mem",
        "proc_alloc",
        "main",
        0,
        0,
        &vec!["tests/spinlock_stress_test.wat".to_owned(); 10000],
    );
    assert_eq!(*results.iter().max().unwrap(), 500 * 10000);
}
