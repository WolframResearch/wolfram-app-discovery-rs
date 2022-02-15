//! Discovery local installations of the Wolfram Language and Wolfram products.

#![warn(missing_docs)]


pub mod config;

mod os;

#[doc(hidden)]
mod test_readme {
    // Ensure that doc tests in the README.md file get run.
    #![doc = include_str!("../README.md")]
}


use std::{fmt, path::PathBuf, process};

use crate::{config::get_env_var, os::OperatingSystem};

//======================================
// Types
//======================================

/// A local installation of the Wolfram System.
#[rustfmt::skip]
#[derive(Debug, Clone)]
pub struct WolframApp {
    //-----------------------
    // Application properties
    //-----------------------
    #[allow(dead_code)]
    app_name: String,
    app_type: WolframAppType,
    app_version: AppVersion,

    app_directory: PathBuf,

    app_executable: Option<PathBuf>,

    // If this is a Wolfram Engine application, then it contains an embedded Wolfram
    // Player application that actually contains the WL system content.
    embedded_player: Option<Box<WolframApp>>,
}

/// Standalone application type distributed by Wolfram Research.
#[derive(Debug, Clone, PartialEq, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "cli", derive(clap::ArgEnum))]
pub enum WolframAppType {
    /// [Wolfram Mathematica](https://www.wolfram.com/mathematica/)
    Mathematica,
    /// [Wolfram Engine](https://wolfram.com/engine)
    Engine,
    /// [Wolfram Desktop](https://www.wolfram.com/desktop/)
    Desktop,
    /// [Wolfram Player](https://www.wolfram.com/player/)
    Player,
    /// [Wolfram Player Pro](https://www.wolfram.com/player-pro/)
    #[doc(hidden)]
    PlayerPro,
    /// [Wolfram Finance Platform](https://www.wolfram.com/finance-platform/)
    FinancePlatform,
    /// [Wolfram Programming Lab](https://www.wolfram.com/programming-lab/)
    ProgrammingLab,
    /// [Wolfram|Alpha Notebook Edition](https://www.wolfram.com/wolfram-alpha-notebook-edition/)
    WolframAlphaNotebookEdition,
    // NOTE: When adding a new variant here, be sure to update WolframAppType::variants().
}

/// Wolfram application version number.
///
/// The major, minor, and revision components of most Wolfram applications will
/// be the same as version of the Wolfram Language they provide.
#[derive(Debug, Clone)]
pub struct AppVersion {
    major: u32,
    minor: u32,
    revision: u32,
    minor_revision: Option<u32>,

    build_code: u32,
}

/// Wolfram Language version number.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WolframVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

#[doc(hidden)]
pub struct Filter {
    pub app_types: Option<Vec<WolframAppType>>,
}

/// Wolfram app discovery error.
#[derive(Debug)]
pub struct Error(String);

impl std::error::Error for Error {}

//======================================
// Functions
//======================================

/// Discover all installed Wolfram applications.
///
/// # Caveats
///
/// This function will use operating-system specific logic to discover installations of
/// Wolfram applications. If a Wolfram application is installed to a non-standard
/// location, it may not be discoverable by this function.
pub fn discover() -> Vec<WolframApp> {
    os::discover_all()
}

/// Discover all installed Wolfram applications that match the specified filtering
/// parameters.
///
/// # Caveats
///
/// This function will use operating-system specific logic to discover installations of
/// Wolfram applications. If a Wolfram application is installed to a non-standard
/// location, it may not be discoverable by this function.
pub fn discover_with_filter(filter: &Filter) -> Vec<WolframApp> {
    let mut apps = os::discover_all();

    apps.retain(|app| filter.check_app(&app).is_ok());

    apps
}

/// Returns the [`$SystemID`][ref/$SystemID] value of the system this code was built for.
///
/// This does require access to a Wolfram Language evaluator.
///
/// [ref/$SystemID]: https://reference.wolfram.com/language/ref/$SystemID.html
// TODO: What exactly does this function mean if the user tries to cross-compile a
//       library?
// TODO: Use env!("TARGET") here and just call system_id_from_target()?
// TODO: Add an `enum SystemID` and use it here. It should have an
//         `as_str(&self) -> &'static str`
//       method.
pub fn target_system_id() -> &'static str {
    match system_id_from_target(env!("TARGET")) {
        Ok(system_id) => system_id,
        Err(err) => panic!(
            "target_system_id() has not been implemented for the current target: {err}"
        ),
    }
}

/// Returns the System ID value that corresponds to the specified Rust
/// [target triple](https://doc.rust-lang.org/nightly/rustc/platform-support.html), if
/// any.
pub fn system_id_from_target(rust_target: &str) -> Result<&'static str, Error> {
    let id = match rust_target {
        // 64-bit x86
        "x86_64-apple-darwin" => "MacOSX-x86-64",
        "x86_64-unknown-linux-gnu" => "Linux-x86-64",
        "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "Windows-x86-64",
        // 64-bit ARM
        "aarch64-apple-darwin" => "MacOSX-ARM64",
        "aarch64-apple-ios" | "aarch64-apple-ios-sim" => "iOS-ARM64", // iOS
        "aarch64-linux-android" => "Android",

        // 32-bit ARM (e.g. Raspberry Pi)
        "armv7-unknown-linux-gnueabihf" => "Linux-ARM",
        _ => {
            return Err(Error(format!(
                "no System ID value associated with Rust target triple: {}",
                rust_target
            )))
        },
    };

    Ok(id)
}

//======================================
// Struct Impls
//======================================

impl WolframAppType {
    /// Enumerate all `WolframAppType` variants.
    pub fn variants() -> Vec<WolframAppType> {
        use WolframAppType::*;

        vec![
            Mathematica,
            Desktop,
            Engine,
            Player,
            PlayerPro,
            FinancePlatform,
            ProgrammingLab,
            WolframAlphaNotebookEdition,
        ]
    }
}

impl WolframVersion {
    /// First component of [`$VersionNumber`][ref/$VersionNumber].
    ///
    /// [ref/$VersionNumber]: https://reference.wolfram.com/language/ref/$VersionNumber.html
    pub fn major(&self) -> u32 {
        self.major
    }

    /// Second component of [`$VersionNumber`][ref/$VersionNumber].
    ///
    /// [ref/$VersionNumber]: https://reference.wolfram.com/language/ref/$VersionNumber.html
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// [`$ReleaseNumber`][ref/$ReleaseNumber]
    ///
    /// [ref/$ReleaseNumber]: https://reference.wolfram.com/language/ref/$ReleaseNumber.html
    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl AppVersion {
    #[allow(missing_docs)]
    pub fn major(&self) -> u32 {
        self.major
    }

    #[allow(missing_docs)]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    #[allow(missing_docs)]
    pub fn revision(&self) -> u32 {
        self.revision
    }

    #[allow(missing_docs)]
    pub fn minor_revision(&self) -> Option<u32> {
        self.minor_revision
    }

    #[allow(missing_docs)]
    pub fn build_code(&self) -> u32 {
        self.build_code
    }
}

impl Filter {
    fn allow_all() -> Self {
        Filter { app_types: None }
    }

    fn check_app(&self, app: &WolframApp) -> Result<(), Error> {
        let Filter { app_types } = self;

        // Filter by application type: Mathematica, Engine, Desktop, etc.
        if let Some(app_types) = app_types {
            if !app_types.contains(&app.app_type()) {
                return Err(Error(format!(
                    "application type '{:?}' is not present in list of filtered app types: {:?}",
                    app.app_type(), app_types
                )));
            }
        }

        Ok(())
    }
}

impl WolframApp {
    /// Evaluate [`$InstallationDirectory`][ref/$InstallationDirectory] using
    /// `wolframscript` to get the location of the local Wolfram Language installation.
    ///
    /// [ref/$InstallationDirectory]: https://reference.wolfram.com/language/ref/$InstallationDirectory.html
    ///
    // TODO: Make this value settable using an environment variable; some people don't
    //       have wolframscript on their `PATH`, or they may have multiple Mathematica
    //       installations and will want to be able to exactly specify which one to use.
    //       WOLFRAM_INSTALLATION_DIRECTORY.
    pub fn try_default() -> Result<Self, Error> {
        WolframApp::try_default_with_filter(&Filter::allow_all())
    }

    #[doc(hidden)]
    pub fn try_default_with_filter(filter: &Filter) -> Result<Self, Error> {
        //------------------------------------------------------------------------
        // If set, use RUST_WOLFRAM_LOCATION (deprecated) or WOLFRAM_APP_DIRECTORY
        //------------------------------------------------------------------------

        if let Some(dir) = config::get_env_default_installation_directory() {
            // TODO: If an error occurs in from_path(), attach the fact that we're using
            //       the environment variable to the error message.
            let app = WolframApp::from_installation_directory(dir)?;

            // If the app doesn't satisfy the filter, return an error. We return an error
            // instead of silently proceeding to try the next discovery step because
            // setting an environment variable constitutes (typically) an explicit choice
            // by the user to use a specific installation. We can't fulfill that choice
            // because it doesn't satisfy the filter, but we can respect it by informing
            // them via an error instead of silently ignoring their choice.
            if let Err(err) = filter.check_app(&app) {
                return Err(Error(format!(
                    "app specified by environment variable does not match filter: {}",
                    err
                )));
            }

            return Ok(app);
        }

        if let Some(dir) = config::get_env_default_app_directory() {
            let app = WolframApp::from_app_directory(dir)?;
            if let Err(err) = filter.check_app(&app) {
                return Err(Error(format!(
                    "app specified by environment variable does not match filter: {}",
                    err
                )));
            }
            return Ok(app);
        }

        //-----------------------------------------------------------------------
        // If wolframscript is on PATH, use it to evaluate $InstallationDirectory
        //-----------------------------------------------------------------------

        if let Some(dir) = try_wolframscript_installation_directory()? {
            let app = WolframApp::from_installation_directory(dir)?;
            // If the app doesn't pass the filter, silently ignore it.
            if !filter.check_app(&app).is_err() {
                return Ok(app);
            }
        }

        //--------------------------------------------------
        // Look in the operating system applications folder.
        //--------------------------------------------------

        let apps: Vec<WolframApp> = discover_with_filter(filter);

        if let Some(first) = apps.into_iter().next() {
            return Ok(first);
        }

        //------------------------------------------------------------
        // No Wolfram applications could be found, so return an error.
        //------------------------------------------------------------

        Err(Error(format!(
            "unable to locate any Wolfram Language installations"
        )))
    }

    /// Construct a `WolframApp` from an application directory path.
    ///
    /// # Example paths:
    ///
    /// Operating system | Example path
    /// -----------------|-------------
    /// macOS            | /Applications/Mathematica.app
    pub fn from_app_directory(app_dir: PathBuf) -> Result<WolframApp, Error> {
        if !app_dir.is_dir() {
            return Err(Error(format!(
                "specified application location is not a directory: {}",
                app_dir.display()
            )));
        }

        os::from_app_directory(&app_dir)?.set_engine_embedded_player()
    }

    /// Construct a `WolframApp` from the
    /// [`$InstallationDirectory`][ref/$InstallationDirectory]
    /// of a Wolfram System installation.
    ///
    /// [ref/$InstallationDirectory]: https://reference.wolfram.com/language/ref/$InstallationDirectory.html
    ///
    /// # Example paths:
    ///
    /// Operating system | Example path
    /// -----------------|-------------
    /// macOS            | /Applications/Mathematica.app/Contents/
    pub fn from_installation_directory(location: PathBuf) -> Result<WolframApp, Error> {
        if !location.is_dir() {
            return Err(Error(format!(
                "invalid Wolfram app location: not a directory: {}",
                location.display()
            )));
        }

        // Canonicalize the $InstallationDirectory to the application directory, then
        // delegate to from_app_directory().
        let app_dir: PathBuf = if cfg!(target_os = "macos") {
            if location.iter().last().unwrap() != "Contents" {
                return Err(Error(format!(
                    "expected last component of installation directory to be \
                    'Contents': {}",
                    location.display()
                )));
            }

            location.parent().unwrap().to_owned()
        } else {
            return Err(platform_unsupported_error(
                "WolframApp::from_installation_directory()",
            ));
        };

        WolframApp::from_app_directory(app_dir)

        // if cfg!(target_os = "macos") {
        //     ... check for .app, application plist metadata, etc.
        //     canonicalize between ".../Mathematica.app" and ".../Mathematica.app/Contents/"
        // }
    }

    // Properties

    /// Get the product type of this application.
    pub fn app_type(&self) -> WolframAppType {
        self.app_type.clone()
    }

    /// Get the application version.
    ///
    /// See also [`WolframApp::wolfram_version()`], which returns the version of the
    /// Wolfram Language bundled with app.
    pub fn app_version(&self) -> &AppVersion {
        &self.app_version
    }

    /// Application directory location.
    pub fn app_directory(&self) -> PathBuf {
        self.app_directory.clone()
    }

    /// Location of the application's main executable.
    ///
    /// * **macOS:** `CFBundleCopyExecutableURL()` location.
    /// * **Windows:** `RegGetValue(_, _, "ExecutablePath", ...)` location.
    /// * **Linux:** *TODO*
    pub fn app_executable(&self) -> Option<PathBuf> {
        self.app_executable.clone()
    }

    /// Returns the version of the Wolfram Language bundled with this application.
    pub fn wolfram_version(&self) -> Result<WolframVersion, Error> {
        if self.app_version.major == 0 {
            return Err(Error(format!(
                "wolfram app has invalid application version: {:?}  (at: {})",
                self.app_version,
                self.app_directory.display()
            )));
        }

        // TODO: Are there any Wolfram products where the application version number is
        //       not the same as the Wolfram Language version it contains?
        //
        //       What about any Wolfram apps that do not contain a Wolfram Languae instance?
        Ok(WolframVersion {
            major: self.app_version.major,
            minor: self.app_version.minor,
            patch: self.app_version.revision,
        })

        /* TODO:
            Look into fixing or working around the `wolframscript` hang on Windows, and generally
            improving this approach. E.g. use WSTP instead of parsing the stdout of wolframscript.

        // MAJOR.MINOR
        let major_minor = self
            .wolframscript_output("$VersionNumber")?
            .split(".")
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        let [major, mut minor]: [String; 2] = match <[String; 2]>::try_from(major_minor) {
            Ok(pair @ [_, _]) => pair,
            Err(major_minor) => {
                return Err(Error(format!(
                    "$VersionNumber has unexpected number of components: {:?}",
                    major_minor
                )))
            },
        };
        // This can happen in major versions, when $VersionNumber formats as e.g. "13."
        if minor == "" {
            minor = String::from("0");
        }

        // PATCH
        let patch = self.wolframscript_output("$ReleaseNumber")?;

        let major = u32::from_str(&major).expect("unexpected $VersionNumber format");
        let minor = u32::from_str(&minor).expect("unexpected $VersionNumber format");
        let patch = u32::from_str(&patch).expect("unexpected $ReleaseNumber format");

        Ok(WolframVersion {
            major,
            minor,
            patch,
        })
        */
    }

    /// The [`$InstallationDirectory`][ref/$InstallationDirectory] of this Wolfram System
    /// installation.
    ///
    /// [ref/$InstallationDirectory]: https://reference.wolfram.com/language/ref/$InstallationDirectory.html
    pub fn installation_directory(&self) -> PathBuf {
        if let Some(ref player) = self.embedded_player {
            return player.installation_directory();
        }

        match OperatingSystem::target_os() {
            OperatingSystem::MacOS => self.app_directory.join("Contents"),
            OperatingSystem::Windows => self.app_directory.clone(),
            // FIXME: Fill this in for Linux
            OperatingSystem::Linux | OperatingSystem::Other => {
                panic!(
                    "{}",
                    platform_unsupported_error("WolframApp::from_app_directory()",)
                )
            },
        }
    }

    //----------------------------------
    // Files
    //----------------------------------

    /// Returns the location of the
    /// [`WolframKernel`](https://reference.wolfram.com/language/ref/program/WolframKernel.html)
    /// executable.
    pub fn kernel_executable_path(&self) -> Result<PathBuf, Error> {
        let path = match OperatingSystem::target_os() {
            OperatingSystem::MacOS => {
                // TODO: In older versions of the product, MacOSX was used instead of MacOS.
                //       Look for either, depending on the version number.
                self.installation_directory()
                    .join("MacOS")
                    .join("WolframKernel")
            },
            OperatingSystem::Windows => {
                self.installation_directory().join("WolframKernel.exe")
            },
            OperatingSystem::Linux | OperatingSystem::Other => {
                return Err(platform_unsupported_error("kernel_executable_path()"));
            },
        };

        if !path.is_file() {
            return Err(Error(format!(
                "WolframKernel executable does not exist in the expected location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    /// Returns the location of the
    /// [`wolframscript`](https://reference.wolfram.com/language/ref/program/wolframscript.html)
    /// executable.
    pub fn wolframscript_executable_path(&self) -> Result<PathBuf, Error> {
        if let Some(ref player) = self.embedded_player {
            return player.wolframscript_executable_path();
        }

        let path = match OperatingSystem::target_os() {
            OperatingSystem::MacOS => PathBuf::from("MacOS").join("wolframscript"),
            OperatingSystem::Windows => PathBuf::from("wolframscript.exe"),
            OperatingSystem::Linux | OperatingSystem::Other => {
                return Err(platform_unsupported_error(
                    "wolframscript_executable_path()",
                ));
            },
        };

        let path = self.installation_directory().join(&path);

        if !path.is_file() {
            return Err(Error(format!(
                "wolframscript executable does not exist in the expected location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    /// Returns the location of the
    /// [`wstp.h`](https://reference.wolfram.com/language/ref/file/wstp.h.html)
    /// header file.
    pub fn wstp_c_header_path(&self) -> Result<PathBuf, Error> {
        let path = self.wstp_compiler_additions_path()?.join("wstp.h");

        if !path.is_file() {
            return Err(Error(format!(
                "wstp.h C header file does not exist in the expected location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    /// Returns the location of the
    /// [WSTP](https://reference.wolfram.com/language/guide/WSTPAPI.html)
    /// static library.
    pub fn wstp_static_library_path(&self) -> Result<PathBuf, Error> {
        let static_archive_name = match OperatingSystem::target_os() {
            // Note: In theory, this can also vary based on the WSTP library 'interface' version
            //       (currently v4). But that has not changed in a long time. If the interface
            //       version does change, this logic should be updated to also check the WL
            //       version.
            OperatingSystem::MacOS => "libWSTPi4.a",
            OperatingSystem::Windows => "wstp64i4s.lib",
            OperatingSystem::Linux | OperatingSystem::Other => {
                return Err(platform_unsupported_error("wstp_static_library_path()"));
            },
        };

        let lib = self
            .wstp_compiler_additions_path()?
            .join(static_archive_name);

        if !lib.is_file() {
            return Err(Error(format!(
                "WSTP static library file does not exist in the expected location: {}",
                lib.display()
            )));
        }

        Ok(lib)
    }

    /// Returns the location of the directory containing the
    /// [Wolfram *LibraryLink*](https://reference.wolfram.com/language/guide/LibraryLink.html)
    /// C header files.
    ///
    /// The standard set of *LibraryLink* C header files includes:
    ///
    /// * WolframLibrary.h
    /// * WolframSparseLibrary.h
    /// * WolframImageLibrary.h
    /// * WolframNumericArrayLibrary.h
    ///
    /// The `wolfram-library-link` crate provides safe wrappers around the Wolfram
    /// *LibraryLink* interface.
    pub fn library_link_c_includes_path(&self) -> Result<PathBuf, Error> {
        if let Some(ref player) = self.embedded_player {
            return player.library_link_c_includes_path();
        }

        if let Some(path) = get_env_var(config::ENV_INCLUDE_FILES_C) {
            return Ok(PathBuf::from(path));
        }

        let path = self
            .installation_directory()
            .join("SystemFiles")
            .join("IncludeFiles")
            .join("C");

        if !path.is_dir() {
            return Err(Error(format!(
                "LibraryLink C header includes directory does not exist in the expected \
                location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    //----------------------------------
    // Utilities
    //----------------------------------

    fn wstp_compiler_additions_path(&self) -> Result<PathBuf, Error> {
        if let Some(ref player) = self.embedded_player {
            return player.wstp_compiler_additions_path();
        }

        if let Some(path) = get_env_var(config::ENV_WSTP_COMPILER_ADDITIONS_DIR) {
            return Ok(PathBuf::from(path));
        }

        let path = self
            .installation_directory()
            .join("SystemFiles")
            .join("Links")
            .join("WSTP")
            .join("DeveloperKit")
            .join(target_system_id())
            .join("CompilerAdditions");

        if !path.is_dir() {
            return Err(Error(format!(
                "WSTP CompilerAdditions directory does not exist in the expected location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

    #[allow(dead_code)]
    fn wolframscript_output(&self, input: &str) -> Result<String, Error> {
        let mut args = vec!["-code".to_owned(), input.to_owned()];

        args.push("-local".to_owned());
        args.push(self.kernel_executable_path().unwrap().display().to_string());

        wolframscript_output(&self.wolframscript_executable_path()?, &args)
    }
}

//----------------------------------
// Utilities
//----------------------------------

fn platform_unsupported_error(name: &str) -> Error {
    Error(format!(
        "operation '{}' is not yet implemented for this platform",
        name
    ))
}

pub(crate) fn print_platform_unimplemented_warning(op: &str) {
    eprintln!(
        "warning: operation '{}' is not yet implemented on this platform",
        op
    )
}

#[cfg_attr(target_os = "windows", allow(dead_code))]
fn warning(message: &str) {
    eprintln!("warning: {}", message)
}

fn wolframscript_output(
    wolframscript_command: &PathBuf,
    args: &[String],
) -> Result<String, Error> {
    let output: process::Output = process::Command::new(wolframscript_command)
        .args(args)
        .output()
        .expect("unable to execute wolframscript command");

    // NOTE: The purpose of the 2nd clause here checking for exit code 3 is to work around
    //       a mis-feature of wolframscript to return the same exit code as the Kernel.
    // TODO: Fix the bug in wolframscript which makes this necessary and remove the check
    //       for `3`.
    if !output.status.success() && output.status.code() != Some(3) {
        panic!(
            "wolframscript exited with non-success status code: {}",
            output.status
        );
    }

    let stdout = match String::from_utf8(output.stdout.clone()) {
        Ok(s) => s,
        Err(err) => {
            panic!(
                "wolframscript output is not valid UTF-8: {}: {}",
                err,
                String::from_utf8_lossy(&output.stdout)
            );
        },
    };

    let first_line = stdout
        .lines()
        .next()
        .expect("wolframscript output was empty");

    Ok(first_line.to_owned())
}

/// If `wolframscript` is available on the users PATH, use it to evaluate
/// `$InstallationDirectory` to locate the default Wolfram Language installation.
///
/// If `wolframscript` is not on PATH, return `Ok(None)`.
fn try_wolframscript_installation_directory() -> Result<Option<PathBuf>, Error> {
    use std::process::Command;

    // Use `wolframscript` if it's on PATH.
    let wolframscript = PathBuf::from("wolframscript");

    // Run `wolframscript -h` to test whether `wolframscript` exists. `-h` because it
    // should never fail, never block, and only ever print to stdout.
    if let Err(err) = Command::new(&wolframscript).args(&["-h"]).output() {
        if err.kind() == std::io::ErrorKind::NotFound {
            // wolframscript executable is not available on PATH
            return Ok(None);
        } else {
            return Err(Error(format!("unable to launch wolframscript: {}", err)));
        }
    };

    // FIXME: Check if `wolframscript` is on the PATH first. If it isn't, we should
    //        give a nicer error message.
    let location = wolframscript_output(
        &wolframscript,
        &["-code".to_owned(), "$InstallationDirectory".to_owned()],
    )?;

    Ok(Some(PathBuf::from(location)))
}

impl WolframApp {
    /// If `app` represents a Wolfram Engine app, set the `embedded_player` field to be
    /// the WolframApp representation of the embedded Wolfram Player.app that backs WE.
    fn set_engine_embedded_player(mut self) -> Result<Self, Error> {
        if self.app_type() != WolframAppType::Engine {
            return Ok(self);
        }

        let embedded_player_path = match OperatingSystem::target_os() {
            OperatingSystem::MacOS => self
                .app_directory
                .join("Contents")
                .join("Resources")
                .join("Wolfram Player.app"),
            OperatingSystem::Windows
            | OperatingSystem::Linux
            | OperatingSystem::Other => {
                // TODO: Does Wolfram Engine on Linux/Windows contain an embedded Wolfram Player,
                //       or is that only done on macOS?
                print_platform_unimplemented_warning(
                    "determine Wolfram Engine path to embedded Wolfram Player",
                );

                // On the hope that returning `app` is more helpful than returning an error here,
                // do that.
                return Ok(self);
            },
        };

        // TODO: If this `?` propagates an error
        let embedded_player = match WolframApp::from_app_directory(embedded_player_path) {
            Ok(player) => player,
            Err(err) => {
                return Err(Error(format!(
                "Wolfram Engine application does not contain Wolfram Player.app in the \
                expected location: {}",
                err
            )))
            },
        };

        self.embedded_player = Some(Box::new(embedded_player));

        Ok(self)
    }
}

//======================================
// Formatting Impls
//======================================

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Error(message) = self;

        write!(f, "Wolfram app error: {}", message)
    }
}

impl fmt::Display for WolframVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let WolframVersion {
            major,
            minor,
            patch,
        } = *self;

        write!(f, "{}.{}.{}", major, minor, patch)
    }
}
