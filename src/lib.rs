//! Discovery local installations of the Wolfram Language and Wolfram products.

#![warn(missing_docs)]


pub mod config;
mod find;

#[doc(hidden)]
mod test_readme {
    // Ensure that doc tests in the README.md file get run.
    #![doc = include_str!("../README.md")]
}


use std::{fmt, path::PathBuf, process, str::FromStr};

use cfg_if::cfg_if;

use crate::config::get_env_var;


//======================================
// Types
//======================================

/// A local installation of the Wolfram System.
#[derive(Debug)]
pub struct WolframApp {
    product: WolframProduct,

    app_directory: PathBuf,

    // If this is a Wolfram Engine application, then it contains an embedded Wolfram
    // Player application that actually contains the WL system content.
    embedded_player: Option<Box<WolframApp>>,
}

/// Wolfram Language version number.
#[non_exhaustive]
pub struct WolframVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

/// Wolfram app discovery error.
#[derive(Debug)]
pub struct Error(String);

/// Standalone product type distributed by Wolfram Research.
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "cli", derive(clap::ArgEnum))]
pub enum WolframProduct {
    /// [Wolfram Mathematica](https://www.wolfram.com/mathematica/)
    Mathematica,
    /// [Wolfram Engine](https://wolfram.com/engine)
    Engine,
    /// [Wolfram Desktop](https://www.wolfram.com/desktop/)
    Desktop,
    /// [Wolfram Player](https://www.wolfram.com/player/)
    Player,
}

impl std::error::Error for Error {}

//======================================
// Functions
//======================================

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
    cfg_if![
        if #[cfg(all(target_os = "macos", target_arch = "x86_64"))] {
            const SYSTEM_ID: &str = "MacOSX-x86-64";
        } else {
            // FIXME: Update this to include common Linux/Windows (and ARM macOS)
            //        platforms.
            compile_error!("target_system_id() has not been implemented for the current system")
        }
    ];

    SYSTEM_ID
}

/// Returns the System ID value that corresponds to the specified Rust
/// [target triple](https://doc.rust-lang.org/nightly/rustc/platform-support.html), if
/// any.
pub fn system_id_from_target(rust_target: &str) -> Result<&'static str, Error> {
    let id = match rust_target {
        "x86_64-apple-darwin" => "MacOSX-x86-64",
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
        //------------------------------------------------------------------------
        // If set, use RUST_WOLFRAM_LOCATION (deprecated) or WOLFRAM_APP_DIRECTORY
        //------------------------------------------------------------------------

        if let Some(dir) = config::get_env_default_installation_directory() {
            // TODO: If an error occurs in from_path(), attach the fact that we're using
            //       the environment variable to the error message.
            return WolframApp::from_installation_directory(dir);
        }

        if let Some(dir) = config::get_env_default_app_directory() {
            return WolframApp::from_app_directory(dir);
        }

        //-----------------------------------------------------------------------
        // If wolframscript is on PATH, use it to evaluate $InstallationDirectory
        //-----------------------------------------------------------------------

        if let Some(dir) = try_wolframscript_installation_directory()? {
            return WolframApp::from_installation_directory(dir);
        }

        //--------------------------------------------------
        // Look in the operating system applications folder.
        //--------------------------------------------------

        // FIXME: Certain kinds of IO errors will cause this to fail, even though we
        //        might successfully find an app if we were to continue looking. E.g. if
        //        reading one particular application fails with a permissions error.
        let apps: Vec<WolframApp> =
            crate::find::search_apps_directory().map_err(|io_err: std::io::Error| {
                Error(format!("error reading applications directory: {}", io_err))
            })?;

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

        let file_name = match app_dir.file_name() {
            Some(file_name) => file_name,
            None => {
                return Err(Error(format!(
                    "specified application location is missing file name component: {}",
                    app_dir.display()
                )))
            },
        };

        let file_name = match file_name.to_str() {
            Some(name) => name,
            None => {
                return Err(Error(format!(
                    "specified application location is not encoded in UTF-8: {}",
                    app_dir.display()
                )))
            },
        };

        if cfg!(target_os = "macos") {
            // TODO: This is possibly too restrictive?
            if !file_name.ends_with(".app") {
                return Err(Error(format!(
                    "expected application directory name to end with .app: {}",
                    app_dir.display()
                )));
            }

            let product = match WolframProduct::try_from_app_file_name(file_name) {
                Some(product) => product,
                None => {
                    return Err(Error(format!(
                        "unrecognized Wolfram application name: {}",
                        file_name
                    )));
                },
            };

            Ok(set_engine_embedded_player(WolframApp {
                product,
                app_directory: app_dir,
                embedded_player: None,
            })?)
        } else {
            Err(platform_unsupported_error(
                "WolframApp::from_app_directory()",
            ))
        }
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
    pub fn product(&self) -> WolframProduct {
        self.product.clone()
    }

    /// The [`$InstallationDirectory`][ref/$InstallationDirectory] of this Wolfram System
    /// installation.
    ///
    /// [ref/$InstallationDirectory]: https://reference.wolfram.com/language/ref/$InstallationDirectory.html
    pub fn installation_directory(&self) -> PathBuf {
        if let Some(ref player) = self.embedded_player {
            return player.installation_directory();
        }

        if cfg!(target_os = "macos") {
            self.app_directory.join("Contents")
        } else {
            // FIXME: Fill this in for Windows and Linux
            panic!(
                "{}",
                platform_unsupported_error("WolframApp::from_app_directory()",)
            )
        }
    }

    /// Returns the Wolfram Language version number of this Wolfram installation.
    pub fn wolfram_version(&self) -> Result<WolframVersion, Error> {
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
    }

    //----------------------------------
    // Files
    //----------------------------------

    /// Returns the location of the
    /// [`WolframKernel`](https://reference.wolfram.com/language/ref/program/WolframKernel.html)
    /// executable.
    pub fn kernel_executable_path(&self) -> Result<PathBuf, Error> {
        let path = if cfg!(target_os = "macos") {
            // TODO: In older versions of the product, MacOSX was used instead of MacOS.
            //       Look for either, depending on the version number.
            self.installation_directory()
                .join("MacOS")
                .join("WolframKernel")
        } else {
            return Err(platform_unsupported_error("kernel_executable_path()"));
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

        let path = if cfg!(target_os = "macos") {
            PathBuf::from("MacOS").join("wolframscript")
        } else {
            return Err(platform_unsupported_error(
                "wolframscript_executable_path()",
            ));
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
        let static_archive_name = if cfg!(target_os = "macos") {
            "libWSTPi4.a"
        } else {
            return Err(platform_unsupported_error("wstp_static_library_path()"));
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

        let path = if cfg!(target_os = "macos") {
            self.installation_directory()
                .join("SystemFiles/IncludeFiles/C/")
        } else {
            return Err(platform_unsupported_error("library_link_c_includes_path()"));
        };

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
            // // Force a rebuild if the path has changed. This happens when developing WSTP.
            // println!("cargo:rerun-if-changed={}", path.display());
            return Ok(PathBuf::from(path));
        }

        let path = if cfg!(target_os = "macos") {
            self.installation_directory()
                .join("SystemFiles/Links/WSTP/DeveloperKit/")
                .join(target_system_id())
                .join("CompilerAdditions")
        } else {
            return Err(platform_unsupported_error("wstp_compiler_additions_path()"));
        };

        if !path.is_dir() {
            return Err(Error(format!(
                "WSTP CompilerAdditions directory does not exist in the expected location: {}",
                path.display()
            )));
        }

        Ok(path)
    }

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

// If `app` represents a Wolfram Engine app, set the `embedded_player` field to be the
// WolframApp representation of the embedded Wolfram Player.app that backs WE.
fn set_engine_embedded_player(mut app: WolframApp) -> Result<WolframApp, Error> {
    if app.product() != WolframProduct::Engine {
        return Ok(app);
    }

    let embedded_player_path = if cfg!(target_os = "macos") {
        app.app_directory
            .join("Contents")
            .join("Resources")
            .join("Wolfram Player.app")
    } else {
        // TODO: Does Wolfram Engine on Linux/Windows contain an embedded Wolfram Player,
        //       or is that only done on macOS?
        print_platform_unimplemented_warning(
            "determine Wolfram Engine path to embedded Wolfram Player",
        );

        // On the hope that returning `app` is more helpful than returning an error here,
        // do that.
        return Ok(app);
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

    app.embedded_player = Some(Box::new(embedded_player));

    Ok(app)
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
