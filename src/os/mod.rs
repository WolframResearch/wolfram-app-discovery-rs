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

    #[cfg(not(target_os = "macos"))]
    Err(crate::platform_unsupported_error(
        "WolframApp::from_app_directory()",
    ))
}
