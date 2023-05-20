# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]




## [0.4.5] — 2023-05-19

### Fixed

* Fixed `wstp_c_header_path()` and `wstp_static_library_path()` in the
  `wolfram_app_discovery::build_scripts` module returning the path to the
  CompilerAdditions directory instead of, respectively, the path to wstp.h and
  the WSTP static library. ([#58])



## [0.4.4] — 2023-03-27

### Fixed

* Support discovery of older Mathematica versions on Linux. ([#52])

  This fixes one problem described in [#51].



## [0.4.3] — 2023-02-03

### Added

* Added logging output along discovery success and error code paths using the
  [`log`](https://crates.io/crates/log) crate logging facade. ([#48])

  Programs making use of wolfram-app-discovery can enable logging in their
  application by initializing a logging implemenation compatible with the `log`
  crate. [`env_logger`](https://crates.io/crates/env_logger) is a common choice
  for logging that can be customized via the `RUST_LOG` environment variable.

  **Logging in Rust build scripts**

  Rust crate `build.rs` scripts using wolfram-app-discovery are strongly
  encouraged to use `env_logger` to make debugging build script behavior easier.

  Adding logging to a `build.rs` script can be done by adding a dependency on
  `env_logger` to Cargo.toml:

  ```toml
  [build-dependencies]
  env_logger = "0.10.0"
  ```

  and initializing `env_logger` at the beginning of `build.rs/main()`:

  ```rust
  fn main() {
      env_logger::init();

      // ...
  }
  ```

  Logging output can be enabled in subsequent crate builds by executing:

  ```shell
  $ RUST_LOG=trace cargo build
  ```

  *Note that `cargo` will suppress output printed by build scripts by default
  unless the build script fails with an error (which matches the expected usage
  of logging output: it is most useful when something goes wrong). Verbose
  `cargo` output (including logging) can be enabled using `cargo -vv`.*



## [0.4.2] — 2023-02-02

### Fixed

* Workaround issue with Wolfram pre-release builds with app version numbers that
  overflow `u32` version fields. ([#46])



## [0.4.1] — 2023-01-06

### Added

* Add new
  [`.github/workflows/build-executables.yml`](https://github.com/WolframResearch/wolfram-app-discovery-rs/blob/v0.4.1/.github/workflows/build-executables.yml)
  file, which was used to retroactively build precompiled binaries for the
  [v0.4.0 release](https://github.com/WolframResearch/wolfram-app-discovery-rs/releases/tag/v0.4.0)
  of the `wolfram-app-discovery` command-line tool. ([#31], [#32], [#33])

* Improve README.md with new 'CLI Documentation' quick link and
  'Installing wolfram-app-discovery' sections, and other minor link and wording
  changes. ([#34], [#35])

* Make major improvements to the `wolfram-app-discovery` command-line tool. ([#36], [#39])

  - The following options are now supported on the `default`, `list`, and `inspect`
    subcommands:

    * `--property <PROPERTIES>` (alias: `--properties`)
    * `--all-properties`
    * `--format <FORMAT>`

    If `--format csv` is specified, the output will be written in the CSV format.

    If `--property` is specified, only the properties listed as an argument will be
    included in the output.

    If `--all-properties` is specified, all available properties will be included in
    the output.

  - The `default` and `inspect` subcommands now support a `--raw-value <PROPERTY>`
    option, which will cause only the value of the specified property to be
    printed.

    This is useful when using `wolfram-app-discovery` as part of a
    compilation workflow or build script. For example:

    ```shell
    # Get the LibraryLink includes directory
    $ export WOLFRAM_C_INCLUDES=`wolfram-app-discovery default --raw-value library-link-c-includes-directory`

    # Invoke a C compiler and provide the LibraryLink headers location
    $ clang increment.c -I$WOLFRAM_C_INCLUDES -shared -o libincrement
    ```

  See [`docs/CommandLineHelp.md`][CommandLineHelp.md@v0.4.1] for complete
  documentation on the `wolfram-app-discovery` command-line interface.

* Add `/opt/Wolfram/` to list of app search locations used on Linux. ([#41])

### Changed

* Replaced custom logic with a dependency on
  [`clap-markdown`](https://crates.io/crates/clap-markdown),
  and used it to regenerate an improved
  [`docs/CommandLineHelp.md`][CommandLineHelp.md@v0.4.1]. ([#30], [#38])


### Fixed

* Fix spurious warnings generated on macOS when no Wolfram applications of
  a particular `WolframAppType` variant could be discovered. ([#37])

* Fix missing support for Linux in
  `wolfram_app_discovery::build_scripts::wstp_static_library_path()` ([#40])

  This ought to have been fixed in [#28], but copy-pasted code meant the same
  fix needed to be applied in two places, and only one was fixed in #28.

  This was preventing the [`wstp-sys`](https://crates.io/crates/wstp-sys) crate
  from compiling on Linux.



## [0.4.0] — 2022-12-14

### Added

* Added support for app discovery on Linux ([#28])

  This address issue [#27].

  [`discover()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/fn.discover.html)
  will now return all Wolfram apps found in the default installation location
  on Linux (currently just `/usr/local/Wolfram/`).

  [`WolframApp::from_app_directory()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/struct.WolframApp.html#method.from_app_directory)
  can now be used to get information on a Wolfram app installed in a non-standard
  location.

  The following `WolframApp` methods are now supported on Linux:

  - [`WolframApp::installation_directory()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/struct.WolframApp.html#method.installation_directory)
  - [`WolframApp::kernel_executable_path()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/struct.WolframApp.html#method.kernel_executable_path)
  - [`WolframApp::wolframscript_executable_path()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/struct.WolframApp.html#method.wolframscript_executable_path)
  - [`WolframApp::wstp_static_library_path()`](https://docs.rs/wolfram-app-discovery/0.4.0/wolfram_app_discovery/struct.WolframApp.html#method.wstp_static_library_path)

* Added custom logic for determining app metadata, in the absence of an
  available standard OS-provided format or API. At the moment, this consists
  of parsing LICENSE.txt and the WolframKernel script for the application type
  and version number, respectively.

  This is likely more fragile than the implementation methods used on macOS and
  Windows, but necessary and sufficient for the time being to get discovery
  working for the most common use-cases. Future improvements are expected.

### Changed

#### Backwards Incompatible

  - Changed the [`AppVersion::build_code()`] method to return `Option<u32>`
    (was `u32`). ([#28])

### Fixed

  - Fixed an issue with platform unsupported error generated by
    `WolframApp::installation_directory()` incorrectly reporting that the error
    was in `WolframApp::from_app_directory()`. ([#28])

  - Filled in an erroneously incomplete `todo!()` in the `Display` impl for
    `Error`. ([#28])




## [0.3.0] – 2022-09-19

### Added

* Add a new
  [`wolfram_library_link::build_scripts`](https://docs.rs/wolfram-app-discovery/0.3.0/wolfram_app_discovery/build_scripts/index.html)
  submodule. ([#25])

  Functions from this module will be used by the `build.rs` scripts of the
  [`wstp`](https://crates.io/crates/wstp) and
  [`wolfram-library-link`](https://crates.io/crates/wolfram-library-link)
  crates. The current implementation of those scripts relies on
  calling methods on a `WolframApp` instance, which means that they don't work
  when no Wolfram applications are available, even if configuration environment
  variables are manually set to point at the necessary headers and libraries.

  - Add new [`Discovery`](https://docs.rs/wolfram-app-discovery/0.3.0/wolfram_app_discovery/build_scripts/enum.Discovery.html) type. ([#25])

### Changed

* Remove unnecessary warning about embedded Wolfram Player. ([#24])

#### Backwards Incompatible

* Change `WolframApp` methods that previously would check an environment
  variable to check only within the app installation directory. ([#25])

  The original usecase for these functions was to get the file paths of the
  LibraryLink and WSTP header files and compiled libraries, for use in the
  build.rs scripts of the `wstp` and `wolfram-library-link` crates. Because
  build scripts often need to be configurable to use files from non-default
  locations, it seemed to make sense to make the `WolframApp` methods themselves
  also have behavior configurable by environment variables.

  However, that behavior was both a bit unintuitive to explain and document (If
  `WolframApp` represents a specific WL installation, why would its methods
  ever return paths *outside* of that app?), and lacked flexibility for the
  build script usecase.

* Move the environment variable declarations into their own
  [`wolfram_library_link::config::env_vars`](https://docs.rs/wolfram-app-discovery/0.3.0/wolfram_app_discovery/config/env_vars/index.html)
  submodule. ([#25])

* Rename `set_print_cargo_build_script_instructions()` to `set_print_cargo_build_script_directives()`. ([#25])



## [0.2.2] – 2022-03-07

### Added

* Improve crate documentation. ([#22])

  - Add examples to crate root comment
  - Update and expand on `WolframApp::try_default()` doc comment.



## [0.2.1] – 2022-03-02

### Added

* Added Windows support for `WolframApp::from_installation_directory()`.  ([#20])



## [0.2.0] – 2022-02-16

### Added

* Added support for app discovery on Windows  ([#17])
  - Fixed the `wolfram-app-discovery` build on Windows
  - Add app discovery logic based on product identifier look-ups in the Windows registry.
  - Improve maintainability of code that branches based on the operating system.

### Changed

* Improve `discover()` to return apps sorted by version number and feature set
  (e.g. apps that provide a notebook front end are sorted ahead of those that don't, if
   the version numbers are otherwise the same).  ([#18])

### Fixed

* Fixed slow execution of `WolframApp::wolfram_version()` (1-3 seconds) due to
  launching a full Wolfram Language kernel process.  ([#17])



## [0.1.2] – 2022-02-08

### Fixed

* Fix compilation failure on non-macOS platforms.  ([#14])



## [0.1.1] – 2022-02-08

### Added

* Added badges for the crates.io version/link, license, and docs.rs link.  ([#10])

### Changed

* Changes the README.md summary line to be consistent with the Cargo.toml `description`
  field.  ([#10])

### Fixed

* Fix broken `target_system_id()` compilation on Linux and Windows that was preventing
  docs.rs from building the crate.  ([#10]).



## [0.1.0] – 2022-02-08

Initial release of `wolfram-app-discovery`.

### Added

* `WolframApp`, which can be used to query information about installed Wolfram
  applications:

  ```rust
  use wolfram_app_discovery::WolframApp;

  let app = WolframApp::try_default()
    .expect("unable to locate any Wolfram applications");

  // Print the $InstallationDirectory of this Wolfram Language installation:
  println!("$InstallationDirectory: {}", app.installation_directory().display());
  ```

* `$ wolfram-app-discovery` command-line tool:

  ```shell
  $ ./wolfram-app-discovery
  Default Wolfram Language installation:

    Product:                     Mathematica
    Wolfram Language version:    13.0.0
    $InstallationDirectory:      /Applications/Mathematica.app/Contents
  ```

* Semi-automatically generated [docs/CommandLineHelp.md](https://github.com/WolframResearch/wolfram-app-discovery-rs/blob/v0.1.0/docs/CommandLineHelp.md) documentation.




<!-- Link anchors -->
[CommandLineHelp.md@v0.4.1]: https://github.com/WolframResearch/wolfram-app-discovery-rs/blob/v0.4.1/docs/CommandLineHelp.md



[#10]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/10
[#14]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/14
[#17]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/17
[#18]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/18
[#20]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/20

<!-- v0.2.2 -->
[#22]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/20

<!-- v0.3.0 -->
[#24]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/24
[#25]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/25

<!-- v0.4.0 -->
[#27]: https://github.com/WolframResearch/wolfram-app-discovery-rs/issues/27
[#28]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/28

<!-- v0.4.1 -->
[#30]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/30
[#31]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/31
[#32]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/32
[#33]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/33
[#34]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/34
[#35]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/35
[#36]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/36
[#37]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/37
[#38]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/38
[#39]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/39
[#40]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/40
[#41]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/41

<!-- v0.4.2 -->
[#46]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/46

<!-- v0.4.3 -->
[#48]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/48

<!-- v0.4.4 -->
[#51]: https://github.com/WolframResearch/wolfram-app-discovery-rs/issues/51
[#52]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/52

<!-- v0.4.5 -->
[#58]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/58


<!-- This needs to be updated for each tagged release. -->
[Unreleased]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.5...HEAD

[0.4.5]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/releases/tag/v0.1.0