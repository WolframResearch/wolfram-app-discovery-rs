# wolfram-app-discovery

Discovery local installations of the Wolfram Language and Wolfram applications.

This crate provides:

* The `wolfram-app-discovery` library, whose API can be used programmatically from Rust code.
* The `wolfram-app-discovery` executable, which can be used from the command-line.

## API Example

Locate the default Wolfram Language installation on this computer:
```rust
use wolfram_app_discovery::WolframApp;

let app = WolframApp::try_default()
    .expect("unable to locate any Wolfram applications");

// Prints a path like:
//   $InstallationDirectory: /Applications/Mathematica.app/Contents/
println!("$InstallationDirectory: {}", app.installation_directory().display());
```

## Command-line Example

Locate the default Wolfram Language installation on this computer:

```shell
$ ./wolfram-app-discovery
Default Wolfram Language installation:

  Product:                     Mathematica
  Wolfram Language version:    13.0.0
  $InstallationDirectory:      /Applications/Wolfram/Mathematica-13.0.0.app/Contents
```

## Configuration

The default method used to locate a Wolfram Language installation
(`WolframApp::try_default()`) will use the following steps to attempt to locate any local
installations, returning the first one found:

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
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for more information.