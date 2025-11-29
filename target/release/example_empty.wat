(module $example_empty.wasm
  (type (;0;) (func))
  (type (;1;) (func (param i32) (result i32)))
  (type (;2;) (func (result i32)))
  (type (;3;) (func (param i32 i32) (result i32)))
  (import "env" "memory" (memory (;0;) 138 138 shared))
  (func $__wasm_init_memory (type 0)
    (block  ;; label = @1
      (block  ;; label = @2
        (block  ;; label = @3
          (br_table 0 (;@3;) 1 (;@2;) 2 (;@1;)
            (i32.atomic.rmw.cmpxchg
              (i32.const 8978560)
              (i32.const 0)
              (i32.const 1))))
        (memory.fill
          (i32.const 65536)
          (i32.const 0)
          (i32.const 8913024))
        (i32.atomic.store
          (i32.const 8978560)
          (i32.const 2))
        (drop
          (memory.atomic.notify
            (i32.const 8978560)
            (i32.const -1)))
        (br 1 (;@1;)))
      (drop
        (memory.atomic.wait32
          (i32.const 8978560)
          (i32.const 1)
          (i64.const -1)))))
  (func $__apex_wasm_set_regs (type 1) (param i32) (result i32)
    (local i32 i32)
    (local.set 2
      (i32.eqz
        (local.tee 1
          (i32.atomic.rmw8.cmpxchg_u
            (i32.add
              (local.get 0)
              (i32.const 65536))
            (i32.const 0)
            (i32.const 1)))))
    (block  ;; label = @1
      (br_if 0 (;@1;)
        (local.get 1))
      (local.set 1
        (i32.add
          (local.tee 0
            (i32.mul
              (local.get 0)
              (i32.const 69632)))
          (i32.const 135288)))
      (global.set $__stack_pointer
        (local.get 1))
      (local.set 0
        (i32.add
          (local.get 0)
          (i32.const 65664)))
      (global.set $__tls_base
        (local.get 0)))
    (local.get 2))
  (func $__apex_wasm_proc_alloc (type 2) (result i32)
    (local i32 i32)
    (local.set 0
      (i32.const 0))
    (block  ;; label = @1
      (loop  ;; label = @2
        (br_if 1 (;@1;)
          (i32.eq
            (local.tee 1
              (local.get 0))
            (i32.const 128)))
        (local.set 0
          (i32.add
            (local.get 1)
            (i32.const 1)))
        (br_if 0 (;@2;)
          (i32.eqz
            (call $__apex_wasm_set_regs
              (local.get 1))))))
    (i32.lt_u
      (local.get 1)
      (i32.const 128)))
  (func $__apex_wasm_proc_free (type 0)
    (local i32)
    (local.set 0
      (i32.const 65536))
    (if  ;; label = @1
      (global.get $__apex_wasm_proc_ptr)
      (then)
      (else
        (unreachable)))
    (global.set $__stack_pointer
      (i32.const 0))
    (global.set $__tls_base
      (i32.const -4097))
    (i32.atomic.store8
      (i32.add
        (i32.const 0)
        (i32.sub
          (global.get $__apex_wasm_proc_ptr)
          (i32.const 1)))
      (i32.const 0))
    (global.set $__apex_wasm_proc_ptr
      (i32.const 0)))
  (func $__original_main (type 2) (result i32)
    (i32.const 0))
  (func $main (type 3) (param i32 i32) (result i32)
    (call $__original_main))
  (func $dummy (type 0))
  (func $__wasm_call_dtors (type 0)
    (call $dummy)
    (call $dummy))
  (func $__apex_wasm_set_regs.command_export (type 1) (param i32) (result i32)
    (call $__apex_wasm_set_regs
      (local.get 0))
    (call $__wasm_call_dtors))
  (func $__apex_wasm_proc_alloc.command_export (type 2) (result i32)
    (call $__apex_wasm_proc_alloc)
    (call $__wasm_call_dtors))
  (func $__apex_wasm_proc_free.command_export (type 0)
    (call $__apex_wasm_proc_free)
    (call $__wasm_call_dtors))
  (func $main.command_export (type 3) (param i32 i32) (result i32)
    (call $main
      (local.get 0)
      (local.get 1))
    (call $__wasm_call_dtors))
  (table (;0;) 1 1 funcref)
  (global $__stack_pointer (mut i32) (i32.const 65536))
  (global $__tls_base (mut i32) (i32.const 0))
  (global $__apex_wasm_proc_ptr (mut i32) (i32.const 0))
  (export "memory" (memory 0))
  (export "__indirect_function_table" (table 0))
  (export "__apex_wasm_set_regs" (func $__apex_wasm_set_regs.command_export))
  (export "__apex_wasm_proc_alloc" (func $__apex_wasm_proc_alloc.command_export))
  (export "__apex_wasm_proc_free" (func $__apex_wasm_proc_free.command_export))
  (export "main" (func $main.command_export))
  (start $__wasm_init_memory)
  (@custom "name" "\00\13\12example_empty.wasm\01\92\02\0c\00\12__wasm_init_memory\01\14__apex_wasm_set_regs\02\16__apex_wasm_proc_alloc\03\15__apex_wasm_proc_free\04\0f__original_main\05\04main\06\05dummy\07\11__wasm_call_dtors\08#__apex_wasm_set_regs.command_export\09%__apex_wasm_proc_alloc.command_export\0a$__apex_wasm_proc_free.command_export\0b\13main.command_export\074\03\00\0f__stack_pointer\01\0a__tls_base\02\14__apex_wasm_proc_ptr")
  (@custom "producers" "\01\0cprocessed-by\01\05clang\0619.1.7")
  (@custom "target_features" "\06+\07atomics+\0bbulk-memory+\0amultivalue+\0fmutable-globals+\0freference-types+\08sign-ext"))
