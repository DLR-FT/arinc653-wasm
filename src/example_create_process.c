#include "ARINC653-wasm.h"

int main(void) {
  PROCESS_ATTRIBUTE_TYPE pa;
  PROCESS_ID_TYPE pid;
  RETURN_CODE_TYPE err;

  CREATE_PROCESS(&pa, &pid, &err);

  APEX_BYTE msg_buf[256] = "hello #";
  msg_buf[7] = (0 % 10) + '0';
  APEX_INTEGER msg_len = sizeof(msg_buf);
  REPORT_APPLICATION_MESSAGE(msg_buf, msg_len, &err);

  return err;
}
