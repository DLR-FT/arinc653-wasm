#include "ARINC653-wasm.h"

unsigned int fibonacci(unsigned int n) {
  if (n <= 1)
    return 1;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

int main(int argc, char** argv) { return fibonacci(argc); }
