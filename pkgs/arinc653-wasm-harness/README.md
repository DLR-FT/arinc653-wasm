When running something with channels, the channel needs to be defined as a UDP port.
That is done via `udp://<name of the channel>@<ip of where the channel data is received from/send to>:<port>` as an argument to the harness.
Then it can be run, for example with the `example_create_process_advanced.wasm` like so:
`cargo run -- ../../target/release/example_create_process_advanced.wasm -s "udp://test@127.0.0.1:5000"`

To actually receive or send data, `nc(1)` can be used:

- Send: `nc -u 127.0.0.1 5000`
- Receive: `nc -lu 127.0.0.1 5000`
