# Overview

A simple wasmtime configuration with host supplied shared memory that runs each module importing this memory in a separate thread.

# Basic Usage / Example

Within this directory, run

> nix develop ../..#rust

> cargo run -- <MODULE_WAT_FILES_OR_MODULE_WASM_FILES>

For options:

> cargo run -- --help

The following example should calculate `fib(35)` on 128 threads.

> ./fibonacci35_128threads.sh

# Setup

The wasm modules this configuration can run has to adhere to certain rules (there is an example module in `tests`):

- They have to import shared memory from the host with the same module and import name. These names can be set with `--host-module-name` and `--shared-memory-name` flags. The memory type of the modules should match.
- They have to export two functions with the same export name for all modules, `proc_alloc: () -> (i32)` and `main: (i32, i32) -> (i32)` (their export names can be set with `--proc-alloc-name` and `--main-function-name` flags):
  - `proc_alloc` is supposed to initialize the thread control block and stack pointers of the module, and is called before main. It should return 1 on successful initialization.
  - If `proc_alloc` returns 1, `main` is called with the arguments set with flags `--main-argc-value` and `--main-argv-value` (otherwise the thread panics). Its return value is printed after all modules have terminated successfully.
