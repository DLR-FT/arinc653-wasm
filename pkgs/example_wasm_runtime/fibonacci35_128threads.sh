#!/usr/bin/env bash

# cd to dir in which script resides
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null

# temporarily go to project root
pushd -- ../..

# build the wasm
nix develop . --command make target/debug/example_fibonacci35.wat

# go back to the dir this script resides in
popd


# ... and run 128 copies of the test app
count=128

output=()
for ((i=1; i<=count; i++)); do
    output+="../../target/debug/example_fibonacci35.wat "
done

cargo run --release -- ${output[@]}
