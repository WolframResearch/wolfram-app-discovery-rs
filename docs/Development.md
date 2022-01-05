
# Development

## Build the `wolfram-app-discovery` executable

The `wolfram-app-discovery` executable target requires the `"cli"` crate feature to be
enabled:

```shell
$ cargo build --features cli
$ ./target/debug/wolfram-app-discovery
```