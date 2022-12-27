# wolfram-app-discovery

[![Crates.io](https://img.shields.io/crates/v/wolfram-app-discovery.svg)](https://crates.io/crates/wolfram-app-discovery)
![License](https://img.shields.io/crates/l/wolfram-app-discovery.svg)
[![Documentation](https://docs.rs/wolfram-app-discovery/badge.svg)](https://docs.rs/wolfram-app-discovery)

#### [API Documentation](https://docs.rs/wolfram-app-discovery) | [CLI Documentation](./docs/CommandLineHelp.md) | [Changelog](./docs/CHANGELOG.md) | [Contributing](./CONTRIBUTING.md)

## About

Find local installations of the Wolfram Language and Wolfram applications.

This crate provides:

* The `wolfram-app-discovery` library, whose API can be used programmatically from Rust code.
  - [API Documentation](https://docs.rs/wolfram-app-discovery)
* The `wolfram-app-discovery` executable, which can be used from the command-line.
  - [CLI Documentation](./docs/CommandLineHelp.md)
  - [Installing `wolfram-app-discovery`](#installing-wolfram-app-discovery)

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
$ ./wolfram-app-discovery
Default Wolfram Language installation:

  Product:                     Mathematica
  Wolfram Language version:    13.0.0
  $InstallationDirectory:      /Applications/Wolfram/Mathematica-13.0.0.app/Contents
```

See [CommandLineHelp.md](./docs/CommandLineHelp.md) for more information on the
`wolfram-app-discovery` command-line interface.

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

## Installing `wolfram-app-discovery`

Prebuilt executables for the `wolfram-app-discovery` command-line tool are
available for all major platforms, and can be
[**downloaded from the GitHub Releases page**](https://github.com/WolframResearch/wolfram-app-discovery-rs/releases).

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Wolfram application licenses

Wolfram applications are covered by different licensing terms than `wolfram-app-discovery`.

The [Wolfram Engine for Developers](https://wolfram.com/engine) is a free
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
