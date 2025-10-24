# Links

- [Required Services](https://www.aviation-ia.com/support-files/arinc653h)
- [Extended Services](https://www.aviation-ia.com/support-files/arinc653p2h)
- [Wasm C-ABI](https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md)
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

# Legal Matter

Copyright © 2025 Deutsches Zentrum für Luft- und Raumfahrt e.V. (DLR).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
