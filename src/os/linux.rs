use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{AppVersion, Error, WolframApp, WolframAppType};

pub fn discover_all() -> Vec<WolframApp> {
    match do_discover_all() {
        Ok(apps) => apps,
        Err(err) => {
            crate::warning("IO error discovering apps: {err}");
            Vec::new()
        },
    }
}

fn do_discover_all() -> Result<Vec<WolframApp>, std::io::Error> {
    // Wolfram apps on Linux are by default installed to a location with the
    // following structure:
    //
    //     /usr/local/Wolfram/<Mathematica|WolframEngine|...>/<MAJOR.MINOR>/

    // TODO(polish): Are there any other root locations that Wolfram products
    //               are installed to by default on Linux?
    let root = Path::new("/usr/local/Wolfram");

    let mut apps = Vec::new();

    for app_type_dir in fs::read_dir(&root)? {
        let app_type_dir = app_type_dir?.path();

        if !app_type_dir.is_dir() {
            continue;
        }

        for app_version_dir in fs::read_dir(&app_type_dir)? {
            let app_version_dir = app_version_dir?.path();

            if !app_version_dir.is_dir() {
                continue;
            }

            match from_app_directory(&app_version_dir) {
                Ok(app) => apps.push(app),
                Err(err) => todo!("error: {err:?}"),
            }
        }
    }

    Ok(apps)
}

//======================================
// WolframApp from app directory
//======================================

pub fn from_app_directory(path: &PathBuf) -> Result<WolframApp, Error> {
    let (app_type, app_version) = parse_app_info_from_files(path)?;

    Ok(WolframApp {
        app_name: app_type.app_name().to_owned(),
        app_type,
        app_version,

        app_directory: path.clone(),

        app_executable: None,

        embedded_player: None,
    })
}

// TODO(cleanup):
//     This entire function is a very hacky way of getting information about an
//     app on Linux, a platform where there is no OS-required standard for
//     application metadata.
fn parse_app_info_from_files(
    app_directory: &PathBuf,
) -> Result<(WolframAppType, AppVersion), Error> {
    //
    // Parse the app type from the first line of LICENSE.txt
    //

    let license_txt = app_directory.join("LICENSE.txt");

    if !license_txt.is_file() {
        return Err(Error::unexpected_layout("LICENSE.txt file", license_txt));
    }

    let contents: String = std::fs::read_to_string(&license_txt)
        .map_err(|err| Error::other(format!("Error reading LICENSE.txt: {err}")))?;

    // TODO(cleanup): Find a better way of determining the WolframAppType than
    //                parsing LICENSE.txt.
    let app_type = match contents.lines().next() {
        Some("Wolfram Mathematica® License Agreement") => WolframAppType::Mathematica,
        Some("Free Wolfram Engine(TM) for Developers: Terms and Conditions of Use") => WolframAppType::Engine,
        Some("Free Wolfram Engine™ for Developers: Terms and Conditions of Use") => WolframAppType::Engine,
        Some(other) => return Err(Error::other(format!(
            "Unable to determine Wolfram app type from LICENSE.txt: first line was: {other:?}"
        ))),
        None => return Err(Error::other("Unable to determine Wolfram app type from LICENSE.txt: file is empty.".to_owned())),
    };

    //
    // Parse the Wolfram version from the WolframKernel launch script
    //

    let wolfram_kernel = app_directory.join("Executables").join("WolframKernel");

    if !wolfram_kernel.is_file() {
        return Err(Error::unexpected_layout(
            "WolframKernel executable",
            wolfram_kernel,
        ));
    }

    let contents: String = std::fs::read_to_string(&wolfram_kernel).map_err(|err| {
        Error::other(format!("Error reading WolframKernel executable: {err}"))
    })?;

    let app_version = match parse_wolfram_kernel_script_contents(&contents)? {
        Some(app_version) => app_version,
        None => {
            return Err(Error::other(format!(
                "Unable to parse app version from WolframKernel: unexpected file contents"
            )))
        },
    };

    Ok((app_type, app_version))
}

fn parse_wolfram_kernel_script_contents(
    contents: &str,
) -> Result<Option<AppVersion>, Error> {
    let mut lines = contents.lines();

    if lines.next() != Some("#!/bin/sh") {
        return Ok(None);
    }

    if lines.next() != Some("#") {
        return Ok(None);
    }

    let info_line = match lines.next() {
        Some(line) => line,
        None => return Ok(None),
    };

    let components: Vec<&str> = info_line.split(' ').collect();

    let version_string = match components.as_slice() {
        &["#", "", "Mathematica", version_string, "Kernel", "command", "file"] => {
            version_string
        },
        other => return Ok(None),
    };

    let app_version = AppVersion::parse(version_string)?;

    Ok(Some(app_version))
}
