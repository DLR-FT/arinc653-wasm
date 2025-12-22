#include "ARINC653-wasm.h"

SAMPLING_PORT_ID_TYPE sid;

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
    VALIDITY_TYPE val;

    READ_SAMPLING_MESSAGE(sid, msg_buf, &msg_len, &val, &err);
    // report the message
    REPORT_APPLICATION_MESSAGE(msg_buf, msg_len, &err);
    // WRITE_SAMPLING_MESSAGE(sid, msg_buf,msg_len, &err);

    // check if an error occured
    if (err) {
      ERROR_MESSAGE_TYPE msg_buf =
          "caused an error during REPORT_APPLICATION_MESSAGE call";
      RAISE_APPLICATION_ERROR(APPLICATION_ERROR, msg_buf, sizeof(msg_buf),
                              &err);
    }

    // increment the counter
    i += 1;

    // wait for next iteration
    PERIODIC_WAIT(&err);
    if (i == 9) {
      // return;
    }
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

  // CREATE_SAMPLING_PORT("test", 0x1000, SOURCE, 0x1000, &sid, &err);
  CREATE_SAMPLING_PORT("test", 0x1000, DESTINATION, 0x1000, &sid, &err);
  CREATE_PROCESS(&pa, &pid, &err);
  START(pid, &err);
  SET_PARTITION_MODE(NORMAL, &err);
}
