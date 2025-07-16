#include "ARINC653-wasm.h"

// Function prototypes
void cold_start(void);
void warm_start(void);

void warm_start(void) {
  PROCESS_ATTRIBUTE_TYPE pa;
  PROCESS_ID_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);
}

void cold_start(void) {
  // do some cold-start-only intitialization business

  warm_start();
}
