# wolfram-app-discovery

[![Crates.io](https://img.shields.io/crates/v/wolfram-app-discovery.svg)](https://crates.io/crates/wolfram-app-discovery)
![License](https://img.shields.io/crates/l/wolfram-app-discovery.svg)
[![Documentation](https://docs.rs/wolfram-app-discovery/badge.svg)](https://docs.rs/wolfram-app-discovery)

#### [API Documentation](https://docs.rs/wolfram-app-discovery) | [CLI Documentation](./docs/CommandLineHelp.md) | [Changelog](./docs/CHANGELOG.md) | [Contributing](./CONTRIBUTING.md)

## About

Find local installations of the Wolfram Language and Wolfram applications.

This crate provides:

* The `wolfram-app-discovery` Rust crate *([API docs](https://docs.rs/wolfram-app-discovery))*
* The `wolfram-app-discovery` command-line tool *([CLI docs](./docs/CommandLineHelp.md), [Installation](#installing-wolfram-app-discovery))*

## Examples

### Using the API

Locate the default Wolfram Language installation on this computer:
```rust
use wolfram_app_discovery::WolframApp;

let app = WolframApp::try_default()
    .expect("unable to locate any Wolfram applications");

// Prints a path like:
//   $InstallationDirectory: /Applications/Mathematica.app/Contents/
println!("$InstallationDirectory: {}", app.installation_directory().display());
```

See also: [`WolframApp::try_default()`][WolframApp::try_default]

### Using the command-line tool

Locate the default Wolfram Language installation on this computer:

```shell
$ wolfram-app-discovery default
App type:                           Mathematica
Wolfram Language version:           13.1.0
Application directory:              /Applications/Wolfram/Mathematica.app
```

See [CommandLineHelp.md](./docs/CommandLineHelp.md) for more information on the
`wolfram-app-discovery` command-line interface.

### Scenario: Building a LibraryLink library

Suppose you have the following C program that provides a function via the
Wolfram *LibraryLink* interface, which you would like to compile and call from
Wolfram Language:

```c
#include "WolframLibrary.h"

/* Adds one to the input, returning the result  */
DLLEXPORT int increment(
  WolframLibraryData libData,
  mint argc,
  MArgument *args,
  MArgument result
) {
    mint arg = MArgument_getInteger(args[0]);
    MArgument_setInteger(result, arg + 1);
    return LIBRARY_NO_ERROR;
}
```

To successfully compile this program, a C compiler will need to be able to find
the included `"WolframLibrary.h"` header file. We can use `wolfram-app-discovery`
to get the path to the appropriate directory:

```shell
# Get the LibraryLink includes directory
$ export WOLFRAM_C_INCLUDES=`wolfram-app-discovery default --raw-value library-link-c-includes-directory`
```

And then pass that value to a C compiler:

```shell
# Invoke the C compiler
$ clang increment.c -I$WOLFRAM_C_INCLUDES -shared -o libincrement
```

The resulting compiled library can be loaded into Wolfram Language using
[`LibraryFunctionLoad`](https://reference.wolfram.com/language/ref/LibraryFunctionLoad)
and then called:

```wolfram
func = LibraryFunctionLoad["~/libincrement", "increment", {Integer}, Integer];

func[5]  (* Returns 6 *)
```

## Installing `wolfram-app-discovery`

[**Download `wolfram-app-discovery` releases.**](https://github.com/WolframResearch/wolfram-app-discovery-rs/releases)

Precompiled binaries for the `wolfram-app-discovery` command-line tool are
available for all major platforms from the GitHub Releases page.

### Using cargo

`wolfram-app-discovery` can be installed using `cargo`
(the [Rust package manager](https://doc.rust-lang.org/cargo/)) by executing:

```shell
$ cargo install --features=cli wolfram-app-discovery
```

This will install the latest version of
[`wolfram-app-discovery` from crates.io](https://crates.io/crates/wolfram-app-discovery).

## Configuration

The default method used to locate a Wolfram Language installation
([`WolframApp::try_default()`][WolframApp::try_default]) will use the following
steps to attempt to locate any local installations, returning the first one found:

1. The location specified by the `WOLFRAM_APP_DIRECTORY` environment variable, if set.
2. If `wolframscript` is on `PATH`, use it to locate the system installation.
3. Check in the operating system applications directory.

#### Configuration example

Specify a particular Wolfram Language installation to use (on macOS):

```shell
$ export WOLFRAM_APP_DIRECTORY="/Applications/Mathematica.app"
```

This environment variable is checked by both the `wolfram-app-discovery` library and
command-line executable.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Wolfram application licenses

Wolfram applications are covered by different licensing terms than `wolfram-app-discovery`.

[Wolfram Engine Community Edition](https://wolfram.com/engine) is a free
distribution of the Wolfram Language, licensed for personal and non-production use cases.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for more information.

See [**Development.md**](./docs/Development.md) for instructions on how to
perform common development tasks when contributing to this project.

See [*Maintenance.md*](./docs/Maintenance.md) for instructions on how to
maintain this project.


[WolframApp::try_default]: https://docs.rs/wolfram-app-discovery/latest/wolfram_app_discovery/struct.WolframApp.html#method.try_default
