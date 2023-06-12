//! Functions for querying the locations of Wolfram development SDK resources,
//! for use in build scripts.
//!
//! The functions in this module are designed to be used from Cargo build scripts
//! via the Rust API.
// TODO: or from the command-line via the `wolfram-app-discovery config` subcommand.
//!
//! Each function will first check a corresponding environment
//! variable before falling back to look up the path in the optionally specified
//! [`WolframApp`].
//!
//! See Also:
//!
//! * [`crate::config::set_print_cargo_build_script_directives()`]

use std::path::PathBuf;

use log::{info, trace};

#[allow(deprecated)]
use crate::{
    config::{
        self,
        env_vars::{
            WOLFRAM_C_INCLUDES, WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY,
            WSTP_COMPILER_ADDITIONS, WSTP_COMPILER_ADDITIONS_DIRECTORY,
        },
    },
    os::OperatingSystem,
    Error, WolframApp,
};

//======================================
// API
//======================================

/// Discovered resource that can come from either a configuration environment
/// variable or from a [`WolframApp`] installation.
///
/// Use [`Discovery::into_path_buf()`] to get the underlying file system path.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Discovery {
    /// Location came from the [`WolframApp`] passed to the lookup function.
    App(PathBuf),

    /// Location derived from an environment variable.
    Env {
        /// The environment variable that was read from.
        ///
        /// This will be a value from [`crate::config::env_vars`].
        variable: &'static str,

        /// The path that was derived from `variable`.
        ///
        /// This value is not always equal to the value of the environment
        /// variable, path components may have been added or removed.
        path: PathBuf,
    },
}

impl Discovery {
    /// Converts `self` into a [`PathBuf`].
    pub fn into_path_buf(self) -> PathBuf {
        match self {
            Discovery::App(path) => path,
            Discovery::Env { variable: _, path } => path,
        }
    }
}

/// Discover the directory containing the
/// [Wolfram *LibraryLink*](https://reference.wolfram.com/language/guide/LibraryLink.html)
/// C header files.
///
/// The following locations are searched in order:
///
/// 1. The [`WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY`] environment variable
/// 2. *Deprecated:* The [`WOLFRAM_C_INCLUDES`] environment variable
/// 3. If `app` contains a value, [`WolframApp::library_link_c_includes_directory()`].
///
/// The standard set of *LibraryLink* C header files includes:
///
/// * WolframLibrary.h
/// * WolframSparseLibrary.h
/// * WolframImageLibrary.h
/// * WolframNumericArrayLibrary.h
///
/// *Note: The [wolfram-library-link](https://crates.io/crates/wolfram-library-link) crate
/// provides safe Rust bindings to the Wolfram *LibraryLink* interface.*
pub fn library_link_c_includes_directory(
    app: Option<&WolframApp>,
) -> Result<Discovery, Error> {
    trace!("start library_link_c_includes_directory(app={app:?})");

    if let Some(resource) =
        get_env_resource(WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY, false)
    {
        info!("discovered in env: {resource:?}");
        return Ok(resource);
    }

    if let Some(resource) = get_env_resource(WOLFRAM_C_INCLUDES, true) {
        info!("discovered in env: {resource:?}");
        return Ok(resource);
    }

    if let Some(app) = app {
        let path = app.library_link_c_includes_directory()?;

        #[rustfmt::skip]
        info!("discovered in app ({:?}): {}", app.installation_directory().display(), path.display());

        return Ok(Discovery::App(path));
    }

    let err = Error::undiscoverable(
        "LibraryLink C includes directory".to_owned(),
        Some(WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY),
    );

    info!("discovery failed: {err}");

    Err(err)
}

//======================================
// WSTP
//======================================

/// Discover the CompilerAdditions subdirectory of the WSTP SDK.
///
/// The following locations are searched in order:
///
/// 1. The [`WSTP_COMPILER_ADDITIONS_DIRECTORY`] environment variable.
/// 2. *Deprecated:* The [`WSTP_COMPILER_ADDITIONS`] environment variable.
/// 3. If `app` contains a value, [`WolframApp::wstp_compiler_additions_directory()`].
///
/// *Note: The [wstp](https://crates.io/crates/wstp) crate provides safe Rust bindings
/// to WSTP.*
///
/// # Alternatives
///
/// When trying to get the path to the
/// [`wstp.h`](https://reference.wolfram.com/language/ref/file/wstp.h.html)
/// header file, or the WSTP static or dynamic library file, prefer to use
/// the following dedicated functions:
///
/// * [`wstp_c_header_path()`]
pub fn wstp_compiler_additions_directory(
    app: Option<&WolframApp>,
) -> Result<Discovery, Error> {
    trace!("start wstp_compiler_additions_directory(app={app:?})");

    if let Some(resource) = get_env_resource(WSTP_COMPILER_ADDITIONS_DIRECTORY, false) {
        info!("discovered in env: {resource:?}");
        return Ok(resource);
    }

    #[allow(deprecated)]
    if let Some(resource) = get_env_resource(WSTP_COMPILER_ADDITIONS, true) {
        info!("discovered in env: {resource:?}");
        return Ok(resource);
    }

    if let Some(app) = app {
        let path = app.target_wstp_sdk()?.wstp_compiler_additions_directory();

        #[rustfmt::skip]
        info!("discovered in app ({:?}): {}", app.installation_directory().display(), path.display());

        return Ok(Discovery::App(path));
    }

    let err = Error::undiscoverable(
        "WSTP CompilerAdditions directory".to_owned(),
        Some(WSTP_COMPILER_ADDITIONS_DIRECTORY),
    );

    info!("discovery failed: {err}");

    Err(err)
}

/// Discover the
/// [`wstp.h`](https://reference.wolfram.com/language/ref/file/wstp.h.html)
/// header file.
///
/// The following locations are searched in order:
///
/// 1. Location derived from [`wstp_compiler_additions_directory()`].
/// 2. If `app` contains a value, [`WolframApp::wstp_c_header_path()`].
///
/// *Note: The [wstp](https://crates.io/crates/wstp) crate provides safe Rust bindings
/// to WSTP.*
pub fn wstp_c_header_path(app: Option<&WolframApp>) -> Result<Discovery, Error> {
    trace!("start wstp_c_header_path(app={app:?})");

    match wstp_compiler_additions_directory(app)? {
        // If this location came from `app`, unwrap the app and return
        // app.wstp_c_header_path() directly.
        Discovery::App(_) => {
            let app = app.unwrap();
            let path = app.target_wstp_sdk()?.wstp_c_header_path();
            #[rustfmt::skip]
            info!("discovered in app ({:?}): {}", app.installation_directory().display(), path.display());
            return Ok(Discovery::App(path));
        },
        Discovery::Env { variable, path } => {
            let wstp_h = path.join("wstp.h");

            if !wstp_h.is_file() {
                let err = Error::unexpected_env_layout(
                    "wstp.h C header file",
                    variable,
                    path,
                    wstp_h,
                );
                info!("discovery failed: {err}");
                return Err(err);
            }

            let discovery = Discovery::Env {
                variable,
                path: wstp_h,
            };
            info!("discovered in env: {discovery:?}");
            return Ok(discovery);
        },
    }
}

/// Discover the
/// [WSTP](https://reference.wolfram.com/language/guide/WSTPAPI.html)
/// static library.
///
/// 1. Location derived from [`wstp_compiler_additions_directory()`].
/// 2. If `app` contains a value, [`WolframApp::wstp_static_library_path()`].
///
/// *Note: The [wstp](https://crates.io/crates/wstp) crate provides safe Rust bindings
/// to WSTP.*
pub fn wstp_static_library_path(app: Option<&WolframApp>) -> Result<Discovery, Error> {
    trace!("start wstp_static_library_path(app={app:?})");

    let static_archive_name =
        wstp_static_library_file_name(OperatingSystem::target_os())?;

    match wstp_compiler_additions_directory(app)? {
        // If this location came from `app`, unwrap the app and return
        // app.wstp_c_header_path() directly.
        Discovery::App(_) => {
            let app = app.unwrap();
            let path = app.target_wstp_sdk()?.wstp_static_library_path();
            #[rustfmt::skip]
            info!("discovered in app ({:?}): {}", app.installation_directory().display(), path.display());
            return Ok(Discovery::App(path));
        },
        Discovery::Env { variable, path } => {
            let static_lib_path = path.join(static_archive_name);

            if !static_lib_path.is_file() {
                let err = Error::unexpected_env_layout(
                    "WSTP static library file",
                    variable,
                    path,
                    static_lib_path,
                )
                .into();
                info!("discovery failed: {err}");
                return Err(err);
            }

            let discovery = Discovery::Env {
                variable,
                path: static_lib_path,
            };
            info!("discovered in env: {discovery:?}");
            return Ok(discovery);
        },
    }
}

//======================================
// Helpers
//======================================

fn get_env_resource(var: &'static str, deprecated: bool) -> Option<Discovery> {
    if let Some(path) = config::get_env_var(var) {
        if deprecated {
            config::print_deprecated_env_var_warning(var, &path);
        }

        return Some(Discovery::Env {
            variable: var,
            path: PathBuf::from(path),
        });
    }

    None
}

// Note: In theory, this can also vary based on the WSTP library 'interface' version
//       (currently v4). But that has not changed in a long time. If the interface
//       version does change, this logic should be updated to also check the WL
//       version.
pub(crate) fn wstp_static_library_file_name(
    os: OperatingSystem,
) -> Result<&'static str, Error> {
    let static_archive_name = match os {
        OperatingSystem::MacOS => "libWSTPi4.a",
        OperatingSystem::Windows => "wstp64i4s.lib",
        OperatingSystem::Linux => "libWSTP64i4.a",
        OperatingSystem::Other => {
            return Err(Error::platform_unsupported(
                "wstp_static_library_file_name()",
            ));
        },
    };

    Ok(static_archive_name)
}

//======================================
// Tests
//======================================

#[test]
fn test_wstp_c_header_path() {
    use crate::ErrorKind;

    //========================

    std::env::remove_var("WSTP_COMPILER_ADDITIONS_DIRECTORY");

    assert_eq!(
        wstp_c_header_path(None),
        Err(Error(ErrorKind::Undiscoverable {
            resource: "WSTP CompilerAdditions directory".into(),
            environment_variable: Some("WSTP_COMPILER_ADDITIONS_DIRECTORY".into())
        }))
    );

    //========================

    std::env::set_var("WSTP_COMPILER_ADDITIONS_DIRECTORY", std::env::temp_dir());

    assert_eq!(
        wstp_c_header_path(None),
        Err(Error(ErrorKind::UnexpectedEnvironmentValueLayout {
            resource_name: "wstp.h C header file".into(),
            env_var: "WSTP_COMPILER_ADDITIONS_DIRECTORY".into(),
            env_value: PathBuf::from(std::env::temp_dir().to_str().unwrap()),
            derived_path: std::env::temp_dir().join("wstp.h")
        }))
    );

    //========================

    // Set WSTP_COMPILER_ADDITIONS_DIRECTORY to a valid value by assuming we
    // can discovery an installed WolframApp that has one.
    let compiler_additions_dir = WolframApp::try_default()
        .unwrap()
        .target_wstp_sdk()
        .unwrap()
        .wstp_compiler_additions_directory();
    std::env::set_var("WSTP_COMPILER_ADDITIONS_DIRECTORY", &compiler_additions_dir);

    assert_eq!(
        wstp_c_header_path(None),
        Ok(Discovery::Env {
            variable: "WSTP_COMPILER_ADDITIONS_DIRECTORY",
            path: compiler_additions_dir.join("wstp.h")
        })
    );
}
