#include "ARINC653-wasm.h"
int fibonacci(int n) {
  if (n <= 1)
    return 1;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(void) { return fibonacci(35); }