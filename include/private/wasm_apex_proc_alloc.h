#ifndef APEX_WASM_PROCESS_UTILS
#define APEX_WASM_PROCESS_UTILS

/* # Purpose
 *
 * This file contains the custom assembly that prepares a new process's
 * execution environment.
 *
 * The processes within an ARINC 653 partition share the same memory
 * address-space (ARINC 653P1-5 chapter 2.3.4, 3.5). Therefore, multiple
 * processes (as per the definition of the term "Process" in ARINC 653) of a
 * partition are what intuitively is referred to as *threads* nowadays.
 *
 *
 * Two things are needed for them to work:
 *
 *
 * ## Secondary stack
 *
 * WebAssembly's (value-) stack can not be pointed to. However, many languages,
 * such as C, rely on being able to get the address of stack allocated
 * variables. To solve this pickle, LLVM creates a secondary stack within the
 * linear memory, together with a stack pointer global (`__stack_pointer`)
 * holding an address to that area in the linear memory. Variables to which
 * pointers will be needed are allocated on the secondary stack within the
 * linear memory.
 *
 * In a multi-threading scenario, all threads share the same address space.
 * However, they need separate stacks, as they most likely diverge in data and
 * control flow. For this purpose, each thread must use a different area in the
 * linear memory for the secondary stack.
 *
 *
 * ## Thread local storage base
 *
 * In many threaded applications, thread local storage (TLS) is needed. This
 * is another per-thread separate allocation of data in the linear memory. The
 * start of that area is referred to by the `__tls_base` global.
 *
 *
 * # Implementation
 *
 * The correctness of this implementation depends on the code handling
 * allocation and deallocation of secondary stack and TLS to themselves not
 * relying on secondary stack/TLS to be set-up.
 *
 * The overall principle is simple:
 *
 * - `__apex_wasm_proc` structure: contains two byte arrays for secondary stack
 *   and TLS.
 * - `__apex_wasm_proc_slots` array: contains precisely enough instances of
 *   `__apex_wasm_proc` to accomodate for `SYSTEM_LIMIT_NUMBER_OF_PROCESSES`
 *   additional threads.
 * - `__apex_wasm_proc_usage_markers` array: contains booleans tracking whether
 *   a slot within `__apex_wasm_proc_slots` is used (true) or unused (false). It
 *   is of the same length as `__apex_wasm_proc_slots`.
 * - `__apex_wasm_proc_alloc` function: allocates secondary stack and TLS from
 *   `__apex_wasm_proc_slots` and sets up the corresponding global variables
 *   (`__stack_pointer` & `__tls_base`). Returns true on success, false on
 *   failure (when all thread slots are already used). Is itself thread-safe.
 * - `__apex_wasm_proc_free`: frees this thread's secondary stack and TLS. Set's
 *   both `__stack_pointer` & `__tls_base` to astronomically high values to make
 *   it likely that an accidential use of them in a thread whose per-thread data
 *   was already freed traps by the first access to secondary stack or TLS.
 *
 *
 * # Generating the assembly
 *
 * To generate the assembly from this code, run:
 *
 * cat wasm_apex_proc_alloc.h | \
 *   wasm32-unknown-wasi-cc -pthread -x c - -Oz -S -o wasm_apex_proc_alloc.S
 *
 * It is worth to play with the optimization level, as LLVM happily optimizes
 * even on inline assembly.
 */

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// define the maxium number of processes
#ifndef SYSTEM_LIMIT_NUMBER_OF_PROCESSES
#define SYSTEM_LIMIT_NUMBER_OF_PROCESSES 128
#endif

// define the secondary stack size to default to 64KiB (0x10000)
#ifndef APEX_WASM_SS_SIZE
#define APEX_WASM_SS_SIZE 0x10000
#endif

// define the thread local storage (TLS) default size to 4KiB (0x1000)
#ifndef APEX_WASM_TLS_SIZE
#define APEX_WASM_TLS_SIZE 0x1000
#endif

// declare the __apex_asm_proc_ptr global
//
// If this value is zero, that means the thread is currently not initialized via
// this file's methods. Any other value indicates it is initialized. The exact
// semantics (e.g. whether this is a pointer or an index) are not stabilized and
// might depend on the optimization level).
__asm__(".globaltype __apex_wasm_proc_ptr, i32\n"
        "__apex_wasm_proc_ptr:\n");

// (re-)declare the __stack_pointer global (if its not declared alredy)
__asm__(".globaltype __stack_pointer, i32\n");

// (re-)declare the __tls_base global (if its not declared alredy)
__asm__(".globaltype __tls_base, i32\n");

// struct holding the per-thread/per-process global state
struct __apex_wasm_proc {
  // thread local storage
  uint8_t tls[APEX_WASM_TLS_SIZE];

  // secondary stack
  // put after TLS, so that a stack-overflow first invalidates that thread's own
  // TLS
  uint8_t ss[APEX_WASM_SS_SIZE];
};

// array of per-thread global state holding structs
struct __apex_wasm_proc
    __apex_wasm_proc_slots[SYSTEM_LIMIT_NUMBER_OF_PROCESSES];

// True if this slot is currently used
_Atomic _Bool __apex_wasm_proc_usage_markers[SYSTEM_LIMIT_NUMBER_OF_PROCESSES];

// from the `__apex_wasm_proc_slots` allocate the first unused one to host
// this thread's secondary stack and thread local storage
__attribute__((export_name("__apex_wasm_proc_alloc"))) _Bool
__apex_wasm_proc_alloc(void) {
  __asm__(
      // abort if proc_ptr is not zero, an already intialized thread tried to
      // re-allocate. Thats a hard error.
      "global.get __apex_wasm_proc_ptr\n"
      "if\n"
      "unreachable\n"
      "end_if\n"

      "loop\n"

      // check if the current i-th slot is used
      // cmpxchg returns false if slot was vacant and is now allocated to us
      // cmpxchg returns true if slot was already used
      "local.get %[loop_counter]\n" // index
      "i32.const %[false_inst]\n"   // expected
      "i32.const %[true_inst]\n"    // desired
      "i32.atomic.rmw8.cmpxchg_u %[base_address_usage_markers]\n"

      "if\n" // slot was already used

      // increment i by 1
      "i32.const 1\n"
      "local.get %[loop_counter]\n"
      "i32.add\n"
      "local.tee %[loop_counter]\n" // also keeps a copy of i on the stack for
                                    // the next step

      "i32.const %[proc_slots_len]\n" // i must always be smaller than this
      "i32.lt_u\n"                    // if i < proc_slots_len
      "br_if 1\n"                     // jumps to loop head

      "else\n" // slot was unused and is now allocated to us

      // get address to the slot's proc struct
      "i32.const %[base_address_slots]\n" // base pointer to slots array
      "i32.const %[proc_size]\n"    // size of one proc struct/element in array
      "local.get %[loop_counter]\n" // index i into slots array
      "i32.mul\n"                   // multiply element size by index
      "i32.add\n"                   // add byte offset to base pointer
      "local.set %[maybe_our_slot_base_addr]\n" // store allocated proc struct
                                                // address

      // set secondary stack pointer
      //
      // as the stack grows downwards, this must be set to the end of the ss
      // member However, it needs to be 16 byte aligned, so we can not just
      // blindly use the last byte of ss:
      //
      // https://github.com/WebAssembly/tool-conventions/blob/main/BasicCABI.md#the-linear-stack
      "local.get %[maybe_our_slot_base_addr]\n" // proc struct address
      "i32.const %[proc_ss_offset]\n" // offset of ss member in proc struct
      "i32.const %[proc_ss_size]\n"   // size of ss member in proc struct
      "i32.add\n" // add proc struct base address, ss member offset and ss
      "i32.add\n" // member size to get the address of the first byte beyond the
                  // ss member
      "i32.const 1\n"
      "i32.sub\n" // subtract one from proc_ss_size
      "i32.const %[alingment_mask]\n"
      "i32.and\n" // zeroize the last n bits to get 4 bits to get 16 byte
                  // alignment
      "global.set __stack_pointer\n"

      // set tls base pointer
      "local.get %[maybe_our_slot_base_addr]\n"
      "i32.const %[proc_tls_offset]\n"
      "i32.add\n"
      "global.set __tls_base\n"

      // store this processes slot index
      "local.get %[loop_counter]\n"
      "i32.const 1\n"
      "i32.add\n"
      "global.set __apex_wasm_proc_ptr\n"

      // return true
      "i32.const %[true_inst]\n"
      "return\n"

      "end_if\n"

      "end_loop\n" // end loop

      // return false
      "i32.const %[false_inst]\n"
      "return\n"

      ::                     // input registers and immediates
      [loop_counter] "r"(0), // local holding the loop counter
      [maybe_our_slot_base_addr] "r"(
          0), // address to slot allocated for this process, if any
      [proc_slots_len] "i"(SYSTEM_LIMIT_NUMBER_OF_PROCESSES),
      [proc_size] "i"(sizeof(struct __apex_wasm_proc)),
      [proc_ss_size] "i"(sizeof(((struct __apex_wasm_proc *)0)->ss)),
      [proc_ss_offset] "i"(offsetof(struct __apex_wasm_proc, ss)),
      [proc_tls_size] "i"(sizeof(((struct __apex_wasm_proc *)0)->tls)),
      [proc_tls_offset] "i"(offsetof(struct __apex_wasm_proc, tls)),
      [base_address_usage_markers] "i"(&__apex_wasm_proc_usage_markers),
      [base_address_slots] "i"(&__apex_wasm_proc_slots), [true_inst] "i"(true),
      [alingment_mask] "i"(~(15)),
      [false_inst] "i"(false)
      : "memory" // better safe than sorry,
  );

  // per default, we assume failure
  return false;
}

// free this stack
__attribute__((export_name("__apex_wasm_proc_free"))) void
__apex_wasm_proc_free(void) {
  __asm__(
      //
      "global.get __apex_wasm_proc_ptr\n"
      "if\n"
      "else\n"
      "unreachable\n"
      "end_if\n"

      // invalidate stack pointer
      //
      // as the stack grows downwards, setting __stack_pointer to 0 should
      // quickly yield a trap
      "i32.const 0\n"
      "global.set __stack_pointer\n"

      // invalidate TLS base
      //
      // TLS is likely small, but more than a byte in size, therefore we set the
      // invalid base just a APEX_WASM_TLS_SIZE bytes before the last byte in
      // the linear memory
      "i32.const %[invalid_tls_addr]\n"
      "global.set __tls_base\n"

      // mark slot as free
      "i32.const %[marker_base_addr]\n"   // base-address
      "global.get __apex_wasm_proc_ptr\n" // one based index
      "i32.const 1\n"
      "i32.sub\n"
      "i32.add\n"                 // address := baseaddres + index
      "i32.const %[false_inst]\n" // false as a const
      "i32.atomic.store8  0\n"

      // invalidate this processes slot index
      "i32.const 0\n"
      "global.set __apex_wasm_proc_ptr\n"

      :: // input registers and immediates

      [invalid_tls_addr] "i"(
          UINT32_MAX -
          sizeof(((struct __apex_wasm_proc *)0)
                     ->tls)), // a hopefully invalid memory address for TLS
      [marker_base_addr] "p"(&__apex_wasm_proc_usage_markers),
      [false_inst] "i"(false));
}

#endif
