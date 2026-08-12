# Links

- [Required Services](https://www.aviation-ia.com/support-files/arinc653h)
- [Extended Services](https://www.aviation-ia.com/support-files/arinc653p2h)
- [Wasm C-ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md)
- [Wasm Linear Stack Description](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md#the-linear-stack)
- [Example of WebAssembly inline assembly](https://github.com/WebAssembly/wasi-libc/blob/1590774d836c482e1c6e480b122214a01c822b50/libc-top-half/musl/src/env/__init_tls.c#L181-L183)
- [Example of WebAssembly `.s` files](https://github.com/WebAssembly/wasi-libc/blob/1590774d836c482e1c6e480b122214a01c822b50/libc-top-half/musl/src/thread/wasm32/wasi_thread_start.s#L6)

# Rationale

- **Choice**: `<an APEX integer type>` -> `APEX_LONG_INTEGER`
  **Reason**: These placeholders are only used for `*_ID_TYPE`. All of these IDs are only ever
  handed out by the OS. By picking the largest possible type available (64-bit integer), we ensure
  that we are compatible with any possible OS, as we can store any possible ID. For an OS that
  chooses a 32-bit integer, the upper bits just remain untouched.
- **Choice**: import the linear memory (via `--import-memory` linker flag)
  **Reason**: Each partition has multiple processes, which are guaranteed to have a shared address space (ARINC 653 P1-5 chapter 2.3.2). The only way to achieve this is if they have shared linear memory. To cause that, they all need to import the linear memory.
- **Choice**: export the function table (via `--export-table` linker flag)
  **Reason**: In order for the `CREATE_PROCESS` call to succeed, the host environment needs to be able to call a guest environment function identified via an index into said table. Exporting the table ensures that the funcref table is accessible from the host environment.
- **Choice**: Do not use `__funcref_t` for function pointers, e.g. the `ENTRY_POINT` as argument in the `CREATE_ERROR_HANDLER` function.
  **Reason**: `__funcref_t` is not representable in Linear Memory. Hence, it can not become the field of a struct passed by-reference. However, the `PROCESS_ATTRIBUTE_TYPE` struct comprises an `ENTRY_POINT` field holding a function pointer. As `__funcref_t` can not be used there, it is necessary to expose a Wasm table for function pointers. Therefore, any use of `__funcref_t` shall be avoided, in order to keep all function pointer representations consistent. (In theory, `CREATE_ERROR_HANDLER` could employ `__funcref_t`, as the function pointer is passed directly as function argument, however, a unified handling of function pointers is preferred.)
- **Choice**: Do not use `__externref_t` for external resources, e.g. `SAMPLING_PORT_ID` as argument to the `CREATE_SAMPLING_PORT` function.
  **Reason**: `__externref_t` is used to pass references to host environment objects through the guest environment, keeping them opaque to the guest environment. It is however desirable to have these accessible from the guest environment, i.e. for a partition to report back a sampling port's ID. Further, making opaque what APEX discloses, would be a breaking change from the APEX API.
- **Choice**: Use `__wasm__` preprocessor `#define` to conditionally annotate official APEX header functions with `import_module` & `import_name` attributes.
  **Reason**: Clang is the sole compiler, that is capable to compile C to Wasm reasonably. Heaving two headers, one without and one with Wasm specific annotations, is unfavorable.
- **Choice**: Use `RETURN_CODE_TYPE` -> `i32`
  **Reason**: [Chapter 6.7.2.2 of ISO/IEC 9899:2018 ("C17")](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n2310.pdf#subsubsection.6.7.2.2) states that the representation of enum types is implementation dependent. However, there is not much of a choice here, since the [Wasm C ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md) dictates `enum`s to be represented as `i32`. [Chapter 6.7.3.3 of ISO/IEC 9899:2024 ("C24")](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf#subsubsection.6.7.3.3) introduces the possibility of specifying the underlying type for `enum`s, which we intentionally ignore to maintain compatibility with older versions of the C standard.
- **Choice**: Employ manual [`import_module`](https://clang.llvm.org/docs/AttributeReference.html#import-module) & [`import_name`](https://clang.llvm.org/docs/AttributeReference.html#import-name) attributes in C-header file instead of using [WIT](https://component-model.bytecodealliance.org/design/wit.html).
  **Reason**: Although WIT is the preferred approach to generate language bindings, `wit-bindgen` neither guarantees (nor does it) generate APEX compliant C header files (structs, typedefs and foremost function signatures). As APEX is merely used by C, C++ and ADA (all of which falling back to the C ABI as FFI), WIT is neither applicable nor required.
- **Choice**: Return values through pointer function arguments.
  **Reason**: WebAssembly is influenced by modern-ish languages like OCaml & Rust, which in part favor returning even complex data types (like structs) from functions by-value. The APEX interfaces however rely on pass-by-reference (pointer to input and output values) for the majority of arguments. This is aligned with [WebAssembly Tool Conventions' Basic C ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md#function-arguments-and-return-values) which for structs and unions larger than one element also prescribes pass by-reference (denoted _indirect_).

# ARINC 653 Wasm Harness

This repository contains a very simplistic ARINC 653 harness. It currently only provides the bare minimum to verify that processes can start, and that some selected ARINC 653 APEX services are provided.
It is by no means even close to complete or ARINC 653 compliant, it solely serves to have _some_ environment to run the Wasm partitions in.

```bash
# Compile the Wasm partitions
make target/release/example_create_process_advanced.wasm

# Start socat. Sampling ports map to UDP sockets, this allows to receive data from Wasm partitions.
socat -u -v UDP-LISTEN:2301,fork,reuseport /dev/null

# Run the harness.
nix run .\#arinc653-wasm-harness -- \
  --sampling-port udp://test@127.0.0.1:2301 \
  target/release/example_create_process_advanced.wasm
```

# Legal Matter

Copyright © 2025-2026 Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
