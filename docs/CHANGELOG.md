# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


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




[#10]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/10
[#14]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/14
[#17]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/17
[#18]: https://github.com/WolframResearch/wolfram-app-discovery-rs/pull/18

<!-- This needs to be updated for each tagged release. -->
[Unreleased]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.2.0...HEAD

[0.2.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/WolframResearch/wolfram-app-discovery-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/WolframResearch/wolfram-app-discovery-rs/releases/tag/v0.1.0