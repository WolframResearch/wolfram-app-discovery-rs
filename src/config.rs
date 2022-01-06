//! Configuration of `wolfram-app-discovery` behavior.

use std::path::PathBuf;

//======================================
// Environment variable names
//======================================

// ==== Warning! ====
//
// The names of these environment variables are ***part of the public API of the
// wolfram-app-discovery library and executable***. Changing which environment variables
// get checked is a backwards incompatible change!
//
// ==== Warning! ====

// PRE_COMMIT: Rename to WOLFRAM_INSTALLATION_DIRECTORY, check for this as a deprecated
//             environment variable as practice.
/// Name of the environment variable that specifies the default Wolfram installation
/// directory.
#[deprecated(note = "use ENV_WOLFRAM_APP_DIRECTORY")]
pub(crate) const ENV_WOLFRAM_LOCATION: &str = "RUST_WOLFRAM_LOCATION";

/// Name of the environment variable that specifies the default Wolfram application
/// directory.
pub const ENV_WOLFRAM_APP_DIRECTORY: &str = "WOLFRAM_APP_DIRECTORY";

pub(crate) const ENV_WSTP_COMPILER_ADDITIONS_DIR: &str = "WSTP_COMPILER_ADDITIONS";
pub(crate) const ENV_INCLUDE_FILES_C: &str = "WOLFRAM_C_INCLUDES";

//======================================
// Functions
//======================================

pub(crate) fn get_env_default_installation_directory() -> Option<PathBuf> {
    #[allow(deprecated)]
    if let Some(dir) = get_env_var(ENV_WOLFRAM_LOCATION) {
        // This environment variable has been deprecated and will not be checked in
        // a future version of wolfram-app-discovery. Use the
        // config::ENV_WOLFRAM_APP_DIRECTORY environment variable instead.
        eprintln!(
            "warning: use of deprecated environment variable '{}' (value={})",
            ENV_WOLFRAM_LOCATION, dir
        );

        return Some(PathBuf::from(dir));
    }

    // TODO: WOLFRAM_APP_INSTALLATION_DIRECTORY? Is this useful in any situation where
    //       WOLFRAM_APP_DIRECTORY wouldn't be easy to set (e.g. set based on
    //       $InstallationDirectory)?

    None
}

/// Check [`ENV_WOLFRAM_APP_DIRECTORY`] to determine the default Wolfram application.
pub fn get_env_default_app_directory() -> Option<PathBuf> {
    if let Some(text) = get_env_var(ENV_WOLFRAM_APP_DIRECTORY) {
        return Some(PathBuf::from(text));
    }

    None
}

pub(crate) fn get_env_var(var: &'static str) -> Option<String> {
    // TODO: Add cargo feature to enable these print statements, so that
    //       wolfram-app-discovery works better when used in build.rs scripts.
    println!("cargo:rerun-if-env-changed={}", var);
    match std::env::var(var) {
        Ok(string) => Some(string),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(err)) => {
            panic!("value of env var '{}' is not valid unicode: {:?}", var, err)
        }
    }
}
