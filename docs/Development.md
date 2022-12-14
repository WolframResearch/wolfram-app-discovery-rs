
# Development

## Build the `wolfram-app-discovery` executable

The `wolfram-app-discovery` executable target requires the `"cli"` crate feature to be
enabled:

```shell
$ cargo build --features cli
$ ./target/debug/wolfram-app-discovery
```

### Check building on other platforms

Doing a full test of `wolfram-app-discovery` requires actually running it
on each platform. However, it is often useful to test that type checking and
building complete successfully when targeting each of the three operating
systems (macOS, Windows, and Linux) that `wolfram-app-discovery` supports.

Note that when doing these quick "does it build?" tests, testing both x86_64 and
ARM variants of an operating system doesn't provide much additional coverage
beyond checking only one or the other.

**Build for macOS:**

```shell
$ cargo build --target x86_64-apple-darwin
$ cargo build --target -apple-darwin
```

**Build for Windows:**

```shell
$ cargo build --target x86_64-pc-windows-msvc
```

**Build for Linux:**

x86-64:

```shell
$ cargo build --target x86_64-unknown-linux-gnu
$ cargo build --target aarch64-unknown-linux-gnu
```
