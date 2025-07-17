#!/usr/bin/env bash

wasm32-unknown-wasi-clang example_threads.c -lc \
  -D_WASI_EMULATED_PTHREAD -lwasi-emulated-pthread \
  -gen-cdb-fragment-path . -o example_threads.wasm \
  -target wasm32-wasi-threads

echo '[' > compile_commands.json
cat *.c.*.json >> compile_commands.json
echo ']' >> compile_commands.json

rm *.c.*.json

wasmtime run --wasm threads=y --wasi cli=y,threads=y example_threads.wasm
