#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;


use std::path::PathBuf;

use crate::{Error, WolframApp};

pub fn discover_all() -> Vec<WolframApp> {
    #[cfg(target_os = "macos")]
    return macos::discover_all();

    #[cfg(target_os = "windows")]
    return windows::discover_all();

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        crate::print_platform_unimplemented_warning(
            "discover all installed Wolfram applications",
        );

        Vec::new()
    }
}

pub fn from_app_directory(dir: &PathBuf) -> Result<WolframApp, Error> {
    #[cfg(target_os = "macos")]
    return macos::from_app_directory(dir);

    #[cfg(target_os = "windows")]
    return windows::from_app_directory(dir);

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err(crate::platform_unsupported_error(
        "WolframApp::from_app_directory()",
    ))
}

//======================================
// Utilities
//======================================

/// Operating systems supported by supported by `wolfram-app-discovery`.
///
/// This enum and [`OperatingSystem::target_os()`] exist to be a less fragile
/// alternative to code like:
///
/// ```ignore
/// if cfg!(target_os = "macos") {
///     // ...
/// } else if cfg!(target_os = "windows") {
///     // ...
/// } else if cfg!(target_os = "linux") {
///     // ...
/// } else {
///     // Error
/// }
/// ```
///
/// Using an enum ensures that all variants are handled in any place where
/// platform-specific logic is required.
pub(crate) enum OperatingSystem {
    MacOS,
    Windows,
    Linux,
    Other,
}

impl OperatingSystem {
    /// Get the [`OperatingSystem`] value for the platform being targeted by the build
    /// of this Rust code.
    pub fn target_os() -> Self {
        if cfg!(target_os = "macos") {
            OperatingSystem::MacOS
        } else if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else if cfg!(target_os = "linux") {
            OperatingSystem::Linux
        } else {
            OperatingSystem::Other
        }
    }
}
