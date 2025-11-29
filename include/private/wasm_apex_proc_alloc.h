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
 * ## Linear stack
 *
 * WebAssembly's (value-) stack can not be pointed to. However, many languages,
 * such as C, rely on being able to get the address of stack allocated
 * variables. To solve this pickle, LLVM creates a linear stack within the
 * linear memory, together with a stack pointer global (`__stack_pointer`)
 * holding an address to that area in the linear memory. Variables to which
 * pointers will be needed are allocated on the linear stack within the linear
 * memory.
 *
 * In a multi-threading scenario, all threads share the same address space.
 * However, they need separate stacks, as they most likely diverge in data and
 * control flow. For this purpose, each thread must use a different area in the
 * linear memory for the linear stack.
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
 * allocation and deallocation of linear stack and TLS to themselves not
 * relying on linear stack/TLS to be set-up.
 *
 * The overall principle is simple:
 *
 * - `__apex_wasm_proc` structure: contains two byte arrays for linear stack
 *   and TLS.
 * - `__apex_wasm_proc_slots` array: contains precisely enough instances of
 *   `__apex_wasm_proc` to accomodate for `SYSTEM_LIMIT_NUMBER_OF_PROCESSES`
 *   additional threads.
 * - `__apex_wasm_proc_usage_markers` array: contains booleans tracking whether
 *   a slot within `__apex_wasm_proc_slots` is used (true) or unused (false). It
 *   is of the same length as `__apex_wasm_proc_slots`.
 * - `__apex_wasm_proc_alloc` function: allocates linear stack and TLS from
 *   `__apex_wasm_proc_slots` and sets up the corresponding global variables
 *   (`__stack_pointer` & `__tls_base`). Returns true on success, false on
 *   failure (when all thread slots are already used). Is itself thread-safe.
 * - `__apex_wasm_proc_free`: frees this thread's linear stack and TLS. Set's
 *   both `__stack_pointer` & `__tls_base` to astronomically high values to make
 *   it likely that an accidential use of them in a thread whose per-thread data
 *   was already freed traps by the first access to linear stack or TLS.
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
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>

// define the maxium number of processes
#ifndef MAX_PROCS
#define MAX_PROCS 128
#endif

// define the linear stack size to default to 64KiB (0x10000)
#ifndef SS_SIZE
#define SS_SIZE 0x10000
#endif

// define the thread local storage (TLS) default size to 4KiB (0x1000)
#ifndef TLS_SIZE
#define TLS_SIZE 0x1000
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
struct __apex_wasm_proc { uint8_t tls[TLS_SIZE]; uint8_t ss[SS_SIZE]; };
struct __apex_wasm_proc __proc_slots[MAX_PROCS];
_Atomic _Bool __proc_used[MAX_PROCS];
__attribute__((export_name("__apex_wasm_set_regs"))) _Bool
__apex_wasm_set_regs(uintptr_t i) {
  _Bool false_inst = false;
  if (atomic_compare_exchange_strong(&__proc_used[i], &false_inst, true)) {
    volatile void *new_sp = &(__proc_slots[i].ss[SS_SIZE - 8]);
    volatile void *new_tls_base = &(__proc_slots[i].tls);
    __asm__("local.get %0\n" "global.set __stack_pointer\n" ::"r"(new_sp));
    __asm__("local.get %0\n" "global.set __tls_base\n" ::"r"(new_tls_base));
    return true; }
    return false; }

// from the `__apex_wasm_proc_slots` allocate the first unused one to host
// this thread's secondary stack and thread local storage
__attribute__((export_name("__apex_wasm_proc_alloc"))) _Bool
__apex_wasm_proc_alloc(void) {
  for (uintptr_t i = 0; i < MAX_PROCS; i++) {

    // define false as stack var, we need to pass them via pointer

    // BUG we take the address of this local variable, thus implicating that
    // there already is a secondary stack, but the secondary stack was not yet
    // set up!

    // find an unused slot
    if (__apex_wasm_set_regs(i))
    // slot was unused
    {
      return true;
    }
    // slot is already used, check next one
    else
      continue;
  }

  // no free slot was found!
  return false;
}

// free this linear stack
__attribute__((export_name("__apex_wasm_proc_free"))) void
__apex_wasm_proc_free(void) {
  __asm__(
      //
      "global.get __apex_wasm_proc_ptr\n"
      "if\n"
      "else\n"
      "unreachable\n"
      "end_if\n"

      // invalidate linear stack pointer
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
          0xffffffff -
          sizeof(((struct __apex_wasm_proc *)0)
                     ->tls)), // a hopefully invalid memory address for TLS
      [marker_base_addr] "p"(&__proc_used),
      [false_inst] "i"(0));
}

#endif
