#!/usr/bin/env bash
(cd ../.. make target/debug/example_fibonacci35.wat)
count=128

output=""
for ((i=1; i<=count; i++)); do
    output+="../../target/debug/example_fibonacci35.wat "
done

cargo run -- $output