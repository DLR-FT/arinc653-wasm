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
