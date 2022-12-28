
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
$ cargo build --target aarch64-apple-darwin
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

## Manual Testing

There is currently no automated method for testing the `wolfram-app-discovery`
CLI. The listings below attempt to enumerate common and uncommon ways to invoke
the CLI so that they can be tested manually by the developer when changes are
made.

### `wolfram-app-discovery` CLI

##### `wolfram-app-discovery default`

**Typical usage:**

```shell
wolfram-app-discovery default
wolfram-app-discovery default --format csv
wolfram-app-discovery default --all-properties
wolfram-app-discovery default --properties app-type,wolfram-version
```

**Combining format and property options:**

```shell
wolfram-app-discovery default --all-properties --format csv
```

**`--raw-value`:**

```shell
wolfram-app-discovery default --raw-value library-link-c-includes-directory
```

**Malformed argument errors:**

```shell
# ERROR
wolfram-app-discovery default --properties app-type --all-properties

# ERROR
wolfram-app-discovery default --raw-value library-link-c-includes-directory --all-properties
```

##### `wolfram-app-discovery list`

```shell
wolfram-app-discovery list
wolfram-app-discovery list --format csv
wolfram-app-discovery list --all-properties
wolfram-app-discovery list --all-properties --format csv
```