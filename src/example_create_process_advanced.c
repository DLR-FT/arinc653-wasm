#include "ARINC653-wasm.h"
#include <stdio.h>

// Function prototypes
void pp_main(void);
void cold_start(void);
void warm_start(void);

// main periodic process
void pp_main(void) {

  // loop counter
  APEX_UNSIGNED i = 0;

  while (1) {
    // allocate buffer for message
    APEX_BYTE msg_buf[256];

    // write message to buffer
    APEX_INTEGER msg_len = 0;
    msg_len = snprintf((char *)msg_buf, sizeof(msg_buf), "hello #%lu", i);

    // (error) return code
    RETURN_CODE_TYPE err;

    // report the message
    REPORT_APPLICATION_MESSAGE(msg_buf, msg_len, &err);

    // check if an error occured
    if (err) {
      ERROR_MESSAGE_TYPE msg_buf =
          "caused an error during REPORT_APPLICATION_MESSAGE call";
      RAISE_APPLICATION_ERROR(APPLICATION_ERROR, msg_buf, sizeof(msg_len),
                              &err);
    }

    // increment the counter
    i += 1;

    // wait for next iteration
    PERIODIC_WAIT(&err);
  }
}

void warm_start(void) {
  PROCESS_ATTRIBUTE_TYPE pa = {
      .PERIOD = 0,
      .TIME_CAPACITY = 0,
      .ENTRY_POINT = &pp_main,
      .STACK_SIZE = 0x10000, // 64 KiB
      .BASE_PRIORITY = 0,
      .DEADLINE = 0,
      .NAME = "pp_main",
  };
  PROCESS_ID_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);
}

void cold_start(void) {
  // do some cold-start-only intitialization business

  warm_start();
}
