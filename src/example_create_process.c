#include "ARINC653-wasm.h"

// #ifndef  SYSTEM_LIMIT_NUMBER_OF_PARTITIONS
#define SYSTEM_LIMIT_NUMBER_OF_PARTITIONS 32
__asm__(".globaltype __apex_wasm_system_limit_number_of_partitions, i32\n"
        "	.export_name	__apex_wasm_system_limit_number_of_partitions, "
        "__apex_wasm_system_limit_number_of_partitions\n"
        ".data\n"
        "__apex_wasm_system_limit_number_of_partitions:\n"
        ".int32 5\n");

// #endif

int main(void) {
  PROCESS_ATTRIBUTE_TYPE pa;
  PROCESS_ID_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);
}
