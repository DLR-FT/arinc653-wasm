#include "ARINC653-wasm.h"

void warm_start() {
  PROCESS_ATTRIBUTE_TYPE pa;
  PROCESS_INDEX_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);
}

void cold_start() {
  // do some cold-start-only intitialization business

  warm_start();
}
