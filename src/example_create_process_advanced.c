#include "ARINC653-wasm.h"

// Function prototypes
void pp_main(void);

// main periodic process
void pp_main(void) {

  // loop counter
  APEX_UNSIGNED i = 0;

  while (1) {
    // allocate buffer for message
    APEX_BYTE msg_buf[256] = "hello #";
    msg_buf[7] = (i % 10) + '0';

    // write message to buffer
    APEX_INTEGER msg_len = sizeof(msg_buf);

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

int main(void) {
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
