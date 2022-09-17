//! Configuration of `wolfram-app-discovery` behavior.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

//======================================
// Environment variable names
//======================================

// ==== Warning! ====
//
// The names of these environment variables are ***part of the public API of the
// wolfram-app-discovery library and executable***. Changing which environment
// variables get checked is a backwards incompatible change!
// // ==== Warning! ====

/// Environment variables.
pub mod env_vars {
    // TODO: Rename to WOLFRAM_INSTALLATION_DIRECTORY, check for this as a
    //       deprecated environment variable as practice.
    /// *Deprecated:* Use [`WOLFRAM_APP_DIRECTORY`] instead.
    /// Name of the environment variable that specifies the default Wolfram installation
    /// directory.
    #[deprecated(note = "use WOLFRAM_APP_DIRECTORY instead")]
    pub(crate) const RUST_WOLFRAM_LOCATION: &str = "RUST_WOLFRAM_LOCATION";

    /// Name of the environment variable that specifies the default Wolfram application
    /// directory.
    pub const WOLFRAM_APP_DIRECTORY: &str = "WOLFRAM_APP_DIRECTORY";

    /// WSTP `CompilerAdditions` directory
    #[deprecated(note = "use WSTP_COMPILER_ADDITIONS_DIRECTORY instead")]
    pub const WSTP_COMPILER_ADDITIONS: &str = "WSTP_COMPILER_ADDITIONS";

    /// WSTP `CompilerAdditions` directory
    ///
    /// In a typical Wolfram Language installation, this is the
    /// `$InstallationDirectory/SystemFiles/Links/WSTP/DeveloperKit/$SystemID/CompilerAdditions/`
    /// directory.
    pub const WSTP_COMPILER_ADDITIONS_DIRECTORY: &str =
        "WSTP_COMPILER_ADDITIONS_DIRECTORY";

    // /// *Deprecated:* Use [`WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY`] instead.
    // #[deprecated(note = "use WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY instead.")]


    /// Wolfram `$InstallationDirectory/SystemFiles/IncludeFiles/C` directory.
    pub const WOLFRAM_C_INCLUDES: &str = "WOLFRAM_C_INCLUDES";

    /// Directory containing the Wolfram *LibraryLink* C header files.
    ///
    /// In a typical Wolfram Language installation, this is the
    /// `$InstallationDirectory/SystemFiles/IncludeFiles/C/` directory.
    pub const WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY: &str =
        "WOLFRAM_LIBRARY_LINK_C_INCLUDES_DIRECTORY";
}

use self::env_vars::WOLFRAM_APP_DIRECTORY;

static PRINT_CARGO_INSTRUCTIONS: AtomicBool = AtomicBool::new(false);

/// Set whether or not `wolfram-app-discovery` will print
/// `cargo:rerun-if-env-changed=<VAR>` directives.
///
/// Defaults to `false`. The previous value for this configuration is returned.
///
/// If `true`, `wolfram-app-discovery` functions will print:
///
/// ```text
/// cargo:rerun-if-env-changed=<VAR>
/// ```
///
/// each time an environment variable is checked by this library (where `<VAR>` is the
/// name of the environment variable).
///
/// Cargo build scripts are intended to set this variable to `true` to ensure that
/// changes in the build's environment configuration will trigger a rebuild. See the
/// [Build Scripts] section of the Cargo Book for more information.
///
///
/// [Build Scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
pub fn set_print_cargo_build_script_directives(should_print: bool) -> bool {
    PRINT_CARGO_INSTRUCTIONS.swap(should_print, Ordering::SeqCst)
}

fn should_print_cargo_build_script_directives() -> bool {
    PRINT_CARGO_INSTRUCTIONS.load(Ordering::SeqCst)
}

pub(crate) fn get_env_default_installation_directory() -> Option<PathBuf> {
    #[allow(deprecated)]
    if let Some(dir) = get_env_var(env_vars::RUST_WOLFRAM_LOCATION) {
        // This environment variable has been deprecated and will not be checked in
        // a future version of wolfram-app-discovery. Use the
        // WOLFRAM_APP_DIRECTORY environment variable instead.
        print_deprecated_env_var_warning(env_vars::RUST_WOLFRAM_LOCATION, &dir);

        return Some(PathBuf::from(dir));
    }

    // TODO: WOLFRAM_APP_INSTALLATION_DIRECTORY? Is this useful in any situation where
    //       WOLFRAM_APP_DIRECTORY wouldn't be easy to set (e.g. set based on
    //       $InstallationDirectory)?

    None
}

/// Check [`WOLFRAM_APP_DIRECTORY`] to determine the default Wolfram application.
pub(crate) fn get_env_default_app_directory() -> Option<PathBuf> {
    if let Some(text) = get_env_var(WOLFRAM_APP_DIRECTORY) {
        return Some(PathBuf::from(text));
    }

    None
}

//======================================
// Helpers
//======================================

pub(crate) fn print_deprecated_env_var_warning(var: &str, value: &str) {
    let message = format!(
        "wolfram-app-discovery: warning: use of deprecated environment variable '{var}' (value={value:?})",
    );

    // Print to stderr.
    eprintln!("{message}");

    // If this is a cargo build script, print a directive that Cargo will
    // highlight to the user.
    if should_print_cargo_build_script_directives() {
        println!("cargo:warning={message}");
    }
}

pub(crate) fn get_env_var(var: &'static str) -> Option<String> {
    if should_print_cargo_build_script_directives() {
        println!("cargo:rerun-if-env-changed={}", var);
    }

    match std::env::var(var) {
        Ok(string) => Some(string),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(err)) => {
            panic!("value of env var '{}' is not valid unicode: {:?}", var, err)
        },
    }
}
