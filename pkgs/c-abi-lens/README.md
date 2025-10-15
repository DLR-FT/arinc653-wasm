# Overview

This application enables read/write access to data structures stored in a foreign Application
Binary Interface (ABI). It does so by generating getter/setter functions for each field of each data
structure declared in an input header file.

- Assuming
  - access to memory which contains an instance _i_ of a structure _s_.
  - the declaration (as in type) of this structure is available via a `.c` or `.h` file _f_.
  - one compilation target architecture is _t_
  - _i_'s in-memory representation conforms to the _t_ ABI
- Then, this tool can generate a header-only library _l_
  - enabling read & write access to the members/fields of _i_ from any architecture _t'_.
  - providing information regarding size and offset within _s_ of each member/field.
  - providing information on the total size of an instance of _s_.
  - which is freestanding, without dependence on anything except for `stdint.h` and optionally
    `byteswap.h`.
- Caveats:
  - _t_ must be an architecture supported by LLVM/the libclang this tool links against.
  - If _t_ and _t'_ are of different endianness, the `-e/--endianness-swap` flag must be passed to
    this tool.
  - _l_ depends on `stdint.h` (which starting from C99 is part of ISO/IEC 9899).
  - If using the endianness conversion, then the three macros `bswap_16`, `bswap_32` & `bswap_64`
    commonly found in `byteswap.h` are required for _l_.
  - _s_ must not contain nested structs.
  - _s_ must not contain arrays of elements larger than one byte in size.
