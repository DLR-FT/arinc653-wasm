#ifndef APEX_WASM_PROCESS_UTILS
#define APEX_WASM_PROCESS_UTILS

/* # Purpose
 *
 * This file serves as prototyping stage for the custom assembly that prepares a
 * new process's execution environment.
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
 * wasm32-unknown-wasi-cc -pthread <THIS_FILE> -Oz -S -o wasm_apex_proc_alloc.s
 */

#include <stdatomic.h>
#include <stdbool.h>
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

// declare the __apex_asm_proc_idx global
__asm__(".globaltype __apex_wasm_proc_idx, i32\n"
        "__apex_wasm_proc_idx:\n");

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
  for (uintptr_t i = 0; i < SYSTEM_LIMIT_NUMBER_OF_PROCESSES; i++) {

    _Bool swap_unsuccessful = true;

    // Effect of the `i32.atomic.rmw8.cmpxchg_u` operation:
    //
    // Let the stack be:
    //
    //     c3 <- Stack Pointer
    //     c2
    //      i
    //     ...
    //
    //
    // Pop c3, c2, i.
    //
    // Then let:
    //
    //     ea  := i + memarg.offset // address in linear memory
    //     cr  := load(ea)          // current value of memory at address ea
    //     cex := c2                // expected value
    //
    // Then if cr == cex:
    //
    //     cw  := c3
    //     store(ea, cw)
    //
    // Finally:
    //
    //     c1  := cr
    //
    // and push c1 to the stack
    __asm__("local.get %[address]\n"  // ea
            "i32.const %[expected]\n" // c2
            "i32.const %[desired]\n"  // c3
            "i32.atomic.rmw8.cmpxchg_u 0\n"
            "local.set %[previous_value]\n"
            : [previous_value] "=r"(swap_unsuccessful)
            : [address] "p"(&__apex_wasm_proc_slots[i]), [expected] "i"(false),
              [desired] "i"(true));

    // find an unused slot
    if (!swap_unsuccessful)
    // slot was unused
    {

      // set stack pointer
      // take one of the last elements, as the stack grows downwards
      // don't literally take the last element, get some nice alignment on the
      // pointer
      __asm__("local.get %0\n"
              "global.set __stack_pointer\n" ::"p"(
                  &(__apex_wasm_proc_slots[i].ss[APEX_WASM_SS_SIZE - 8])));

      // set tls base
      __asm__("local.get %0\n"
              "global.set __tls_base\n" ::"r"(__apex_wasm_proc_slots[i].tls));

      // // set own process id
      __asm__("local.get %0\n"
              "global.set __apex_wasm_proc_idx\n" ::"r"(i));

      // TODO maye zeroize ss & tls?

      // success!
      return true;
    }
    // slot is already used, check next one
    else
      continue;
  }

  // no free slot was found!
  return false;
}

// free this stack
__attribute__((export_name("__apex_wasm_proc_free"))) void
__apex_wasm_proc_free(void) {
  __asm__(
      // invalidate stack pointer
      "i32.const %[invalid_addr]\n"
      "global.set __stack_pointer\n"

      // invalidate TLS base
      "i32.const %[invalid_addr]\n"
      "global.set __tls_base\n"

      // mark slot as free
      "i32.const %[marker_base_addr]\n"   // base-address
      "global.get __apex_wasm_proc_idx\n" // index
      "i32.add\n"                         // address := baseaddres + index
      "i32.const %[false_inst]\n"         // false as a const
      "i32.atomic.store8  0\n" ::[invalid_addr] "i"(UINT32_MAX),
      [marker_base_addr] "i"(__apex_wasm_proc_usage_markers),
      [false_inst] "i"(false));
}

#endif
