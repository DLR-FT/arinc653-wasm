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
 * Two things are needed for that:
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
 * In many threaded applications, thread local storage is needed. This is
 * another per-thread separate allocation of data in the linear memory. The
 * start of that area is referred to by the `__tls_base` global.
 *
 *
 * # Implementation
 *
 * The correctness of this implemtnation depends on the code handling allocation
 * and deallocation of secondary stack and tls to themselves not relying on
 * secondary stack/tls to be set-up.
 *
 * The overall principle is simple:
 *
 * - `__apex_wasm_thread` structure: contains two byte arrays for secondary
 *   stack and tls.
 * - `__apex_wasm_thread_slots` array: contains precisely enough instances of
 *   `__apex_wasm_thread` to accomodate for `SYSTEM_LIMIT_NUMBER_OF_PROCESSES`
 *   additional threads.
 * - `__apex_wasm_thread_usage_markers` array: contains booleans tracking
 *   whether a slot within `__apex_wasm_thread_slots` is used (true) or unused
 *   (false). It is of the same length as `__apex_wasm_thread_slots`.
 * - `__apex_wasm_proc_alloc` function: allocates secondary stack and tls from
 *   `__apex_wasm_thread_slots` and sets up the corresponding global variables
 *   (`__stack_pointer` & `__tls_base`). Returns true on success, false on
 *   failure (when all thread slots are already used). Is itself thread-safe.
 * - `__apex_wasm_proc_free`: frees this thread's secondary stack and tls. Set's
 *   both `__stack_pointer` & `__tls_base` to astronomically high values to make
 *   it likely that an accidential use of them in a thread whose per-thread data
 *   was already freed traps by the first access to secondary stack or tls.
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

// define the secondary stack (SS) size to default to 64KiB (0x10000)
#ifndef APEX_WASM_SS_SIZE
#define APEX_WASM_SS_SIZE 0x10000
#endif

// define the thread local storage (TLS) default size to 4KiB (0x1000)
#ifndef APEX_WASM_TLS_SIZE
#define APEX_WASM_TLS_SIZE 0x1000
#endif

// declare the __apex_asm_proc_id global
__asm__(".globaltype __apex_wasm_proc_id, i32\n"
        "__apex_wasm_proc_id:\n");

// (re-)declare the __tls_base global (if its not declared alredy)
__asm__(".globaltype __tls_base, i32\n");

// struct holding the per-thread global state
struct __apex_wasm_thread {
  // thread-local-storage
  uint8_t tls[APEX_WASM_TLS_SIZE];

  // secondary stack
  // put after tls, so that a stack-overflow first invalidates that threads own
  // tls
  uint8_t ss[APEX_WASM_SS_SIZE];
};

// array of per-thread global state holding structs
struct __apex_wasm_thread
    __apex_wasm_thread_slots[SYSTEM_LIMIT_NUMBER_OF_PROCESSES];

// True if this slot is currently used
_Atomic _Bool
    __apex_wasm_thread_usage_markers[SYSTEM_LIMIT_NUMBER_OF_PROCESSES];

// from the `__apex_wasm_thread_slots` allocate the first unused one to host
// this thread's secondary stack and thread local storage
__attribute__((export_name("__apex_wasm_proc_alloc"))) _Bool
__apex_wasm_proc_alloc(void) {
  for (uintptr_t i = 0; i < SYSTEM_LIMIT_NUMBER_OF_PROCESSES; i++) {

    // define false as stack var, we need to pass them via pointer
    _Bool false_inst = false;

    // find an unused slot
    if (atomic_compare_exchange_strong(&__apex_wasm_thread_usage_markers[i],
                                       &false_inst, true))
    // slot was unused
    {

      // get volatile pointer to the stack end
      // take one of the last elements, as the stack grows downwards
      // don't literally take the last element, get some nice alignment on the pointer
      volatile void *new_stack_pointer = &(__apex_wasm_thread_slots[i].ss[APEX_WASM_SS_SIZE - 8]);

      // get volatile pointer to the tls start
      volatile void *new_tls_base = &(__apex_wasm_thread_slots[i].tls);

      // set stack pointer
      __asm__("local.get %0\n"
              "global.set __stack_pointer\n" ::"r"(new_stack_pointer));

      // set tls base
      __asm__("local.get %0\n"
              "global.set __tls_base\n" ::"r"(new_tls_base));

      // // set own process id
      __asm__("local.get %0\n"
              "global.set __apex_wasm_proc_id\n" ::"r"(i));

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
  uint32_t my_process_id = 0;

  // get this thead's id
  __asm__("global.get __apex_wasm_proc_id\n"
          "local.set %0\n"
          : "=r"(my_process_id));

  uint32_t invalid_addr = UINT32_MAX;

  // invalidate stack pointer
  __asm__("local.get %0\n"
          "global.set __stack_pointer\n" ::"r"(invalid_addr));

  // invalidate tls base
  __asm__("local.get %0\n"
          "global.set __tls_base\n" ::"r"(invalid_addr));

  // mark thread as free
  atomic_store(&__apex_wasm_thread_usage_markers[my_process_id], false);
}

#endif
