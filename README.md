# Links

- [Required Services](https://www.aviation-ia.com/support-files/arinc653h)
- [Extended Services](https://www.aviation-ia.com/support-files/arinc653p2h)
- [Wasm C-ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md)

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
- **Choice**: Do not use `__externref_t` for function pointers, e.g. the `ENTRY_POINT` argument in the `CREATE_ERROR_HANDLER` function.
  **Reason**: `__externref_t` is not representable in Linear Memory. Hence, it can not become the field of a struct. However, the `PROCESS_ATTRIBUTE_TYPE` struct comprises an `ENTRY_POINT` field holding a function pointer. As `__externref_t` can not be used there, it is necessary to expose the table for function pointers. Therefore, any use of `__externref_t` shall be avoided, in order to keep all function pointer representations consistent.
  **Note**: Passing plain function pointers from guest to host as performed for `CREATE_ERROR_HANDLER` and `CREATE_PROCESS` does not particular follow WebAssembly standards (except threading). However, to stay compliant to ARINC653, such is required.
- **Choice**: Use `__wasm__` preprocessor `#define` to conditionally annotate official APEX header functions with `import_module` & `import_name` attributes.
  **Reason**: Clang is the sole compiler, that is capable to compile C to Wasm reasonably. Heaving two headers, one without and one with Wasm specific annotations, is unfavorable.
- **Choice**: Employ attributes `import_module` & `import_name` in C-header despite of WIT.
  **Reason**: Although WIT is the preferred approach to generate language bindings, `wit-bindgen` neither guarantees (nor does it) generate APEX compliant header defintions (headers, structs, typedefs). As APEX is merely used by C, C++ and ADA, a WIT is not particularly required.
- **Choice**: Return values through function parameters (pointers).
  **Reason**: The WebAssembly standard is motivated by Haskell, Rust that passed values back from function through `return`. As the APEX interfaces all pass solely values back through pointers provided through function parameters, the WebAssembly standard can't be met. ["Each function takes a sequence of values as parameters and returns a sequence of values as results."](https://www.w3.org/TR/wasm-core-2/#concepts%E2%91%A0)

# Legal Matter

Copyright © 2025 Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
