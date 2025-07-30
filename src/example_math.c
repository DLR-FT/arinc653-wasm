#include "ARINC653-wasm.h"
#include <math.h>
#include <string.h>

// Function prototypes
void cold_start(void);
void warm_start(void);

void warm_start(void) {
  volatile float x = 1.6;
  float y;

  y = sin(x);

  if (0.9 < y && y < 1.0) {
    char *msg = "math seems to work";
    RETURN_CODE_TYPE error;
    REPORT_APPLICATION_MESSAGE((MESSAGE_ADDR_TYPE)msg, strlen(msg), &error);
  }
}

void cold_start(void) { warm_start(); }
