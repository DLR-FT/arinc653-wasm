#include "ARINC653-wasm.h"

int main(void) {
  PROCESS_ATTRIBUTE_TYPE pa;
  PROCESS_ID_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);
}

#include "wasm_apex_proc_alloc.h"
