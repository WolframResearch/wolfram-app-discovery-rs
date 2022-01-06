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