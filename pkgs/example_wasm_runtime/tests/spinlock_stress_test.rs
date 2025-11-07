use example_wasm_runtime::run;

#[test]
fn spinlock_stress_test() {
    let count_up_to_500 = r#"(module
  (import "host" "shared_mem" (memory 1 2 shared))

  (func $acquire
    (loop 
        (i32.atomic.rmw.cmpxchg (i32.const 0) (i32.const 0) (i32.const 1))
        (br_if 0)
    )
  )

  (func $release
    (i32.atomic.rmw.cmpxchg (i32.const 0) (i32.const 1) (i32.const 0))
    drop
  )

  (func $increment (result i32)
        (i32.const 8)
        (i32.const 8)
        (i32.load)
        (i32.const 1)
        (i32.add)
        (i32.store)
        (i32.load (i32.const 8))
  )

  (func $proc_alloc (export "proc_alloc") (result i32) i32.const 1)
  (func $main (export "main") (param i32) (param i32) (result i32)
    (local $val i32)
    (local $i i32)
    (local.set $i (i32.const 500))
    (loop
        call $acquire
        call $increment
        local.set $val
        call $release
        (i32.sub (local.get $i) (i32.const 1))
        (local.tee $i)
        br_if 0
    )
    (local.get $val)
  )
)"#;
    let results = run(
        "host",
        "shared_mem",
        "proc_alloc",
        "main",
        0,
        0,
        vec![count_up_to_500.to_owned(); 10000],
    );
    assert_eq!(*results.iter().max().unwrap(), 500 * 10000);
}
