use std::{io, path::PathBuf};

use crate::WolframApp;

/// Search the operating system applications directory.
pub(crate) fn search_apps_directory() -> Result<Vec<WolframApp>, io::Error> {
    if cfg!(target_os = "macos") {
        find_macos_apps()
    } else {
        // FIXME: Implement this functionality for Windows and Linux.
        crate::print_platform_unimplemented_warning("search application directory");
        Ok(Vec::new())
    }
}

fn find_macos_apps() -> Result<Vec<WolframApp>, io::Error> {
    let mut apps: Vec<PathBuf> = Vec::new();

    // Find .app files, recursively.
    for entry in std::fs::read_dir(&PathBuf::from("/Applications"))? {
        let entry = entry?;

        let file_type = entry.file_type()?;

        if !file_type.is_dir() {
            continue;
        }

        // if entry.file_name().
        let file_name = entry.file_name();
        let file_name: &str = match file_name.to_str() {
            Some(name) => name,
            None => {
                eprintln!(
                    "warning: ignoring application with non-UTF-8 file name: {}",
                    file_name.to_string_lossy()
                );
                continue;
            },
        };

        // If this is an .app file, add it's path to `apps`.
        if file_name.ends_with(".app") {
            apps.push(entry.path());
        }
    }

    let mut wolfram_apps: Vec<WolframApp> = Vec::new();

    for app_dir in apps {
        if let Ok(wolfram_app) = WolframApp::from_app_directory(app_dir) {
            wolfram_apps.push(wolfram_app);
        }
    }

    // TODO: Sort by wolfram version number, once that doesn't requiring calling out to
    //       wolframscript (which is slow, ~few seconds per app) to compute.
    Ok(wolfram_apps)
}
