mod cf_exts;

use std::path::PathBuf;

use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    base::CFRelease,
    bundle::{CFBundleCopyExecutableURL, CFBundleCreate, CFBundleRef},
    error::CFErrorRef,
    string::CFStringRef,
    url::CFURLRef,
};

use crate::{AppVersion, Error, WolframApp, WolframAppType};

pub fn discover_all() -> Vec<WolframApp> {
    load_installed_products_from_launch_services()
}

pub fn from_app_directory(path: &PathBuf) -> Result<WolframApp, Error> {
    let url: CFURLRef = match cf_exts::url_create_with_file_system_path(path) {
        Some(url) => url,
        None => {
            return Err(Error::other(format!(
                "unable to create CFURL from path: {:?}",
                path
            )))
        },
    };

    unsafe { get_app_from_url(url, None) }
}

impl WolframAppType {
    #[rustfmt::skip]
    fn bundle_id(&self) -> &'static str {
        use WolframAppType::*;

        match self {
            Mathematica                 => "com.wolfram.Mathematica",
            PlayerPro                   => "com.wolfram.Mathematica.PlayerPro",
            Player                      => "com.wolfram.Mathematica.Player",
            Desktop                     => "com.wolfram.Desktop",
            Engine                      => "com.wolfram.WolframEngine",
            FinancePlatform             => "com.wolfram.FinancePlatform",
            ProgrammingLab              => "com.wolfram.ProgrammingLab",
            WolframAlphaNotebookEdition => "com.wolfram.WolframAlpha.Notebook",
        }
    }
}

unsafe fn get_app_from_url(
    app_url: CFURLRef,
    mut app_type: Option<WolframAppType>,
) -> Result<WolframApp, Error> {
    let bundle: CFBundleRef = CFBundleCreate(std::ptr::null(), app_url);

    if bundle.is_null() {
        return Err(Error::other("invalid CFBundleRef pointer".to_owned()));
    }

    //
    // Get the application bundle identifier
    //

    let bundle_id = match cf_exts::bundle_identifier(bundle) {
        Some(id) => id,
        None => {
            return Err(Error::other(format!(
                "unable to read application bundle identifier"
            )))
        },
    };

    // Sanity check that the app type declared by the caller matches the apps actual
    // bundle identifier.
    if let Some(ref app_type) = app_type {
        assert_eq!(bundle_id, app_type.bundle_id());
    }

    //
    // Get the application type (if not declared already by the caller)
    //

    let app_type: WolframAppType = match app_type {
        Some(type_) => type_,
        None => {
            app_type = WolframAppType::variants().into_iter().find(|app| {
                // Perform a case-insensitive comparison.
                app.bundle_id().to_ascii_lowercase() == bundle_id.to_ascii_lowercase()
            });

            match app_type {
                Some(type_) => type_,
                None => {
                    return Err(Error::other(format!(
                        "application bundle identifier is not a known Wolfram app: {}",
                        bundle_id
                    )))
                },
            }
        },
    };

    //
    // Get the application directory
    //

    let app_directory: PathBuf =
        match cf_exts::url_get_file_system_representation(app_url) {
            Some(path) => path,
            None => {
                return Err(Error::other(format!(
                    "unable to convert application CFURL to file system representation"
                )))
            },
        };

    assert!(app_directory.is_absolute());

    //
    // Get the application main executable
    //

    let exec_url: CFURLRef = CFBundleCopyExecutableURL(bundle);

    let app_executable: Option<PathBuf> = if !exec_url.is_null() {
        let path: PathBuf = match cf_exts::url_get_file_system_representation(exec_url) {
            Some(path) => path,
            None => {
                return Err(Error::other(format!(
                    "unable to convert application executable CFURL to file system \
                    representation"
                )))
            },
        };

        assert!(path.is_absolute());

        CFRelease(exec_url as *const _);
        Some(path)
    } else {
        None
    };

    //
    // Get the application version number
    //

    let app_version = match cf_exts::bundle_get_value_for_info_dictionary_key(
        bundle,
        "CFBundleShortVersionString",
    ) {
        Some(version) => AppVersion::parse(&version).map_err(|err| {
            Error::other(format!(
                "unable to parse application short version string: '{}': {}",
                version, err
            ))
        })?,
        None => {
            return Err(Error::other(format!(
                "unable to read application short version string"
            )))
        },
    };

    let app_name =
        cf_exts::bundle_get_value_for_info_dictionary_key(bundle, "CFBundleName")
            .ok_or_else(|| {
                Error::other("app is missing CFBundleName property".to_owned())
            })?;

    //
    // Release `bundle` and return the final WolframApp description.
    //

    CFRelease(bundle as *const _);

    WolframApp {
        app_type,
        app_name,
        app_directory,
        app_executable,
        app_version,
        embedded_player: None,
    }
    .set_engine_embedded_player()
}

fn load_installed_products_from_launch_services() -> Vec<WolframApp> {
    let mut app_bundles = Vec::new();

    for app_type in WolframAppType::variants() {
        let bundle_id: CFStringRef = cf_exts::cf_string_from_str(app_type.bundle_id());

        unsafe {
            let mut out: CFErrorRef = std::ptr::null_mut();
            let app_urls: CFArrayRef =
                cf_exts::LSCopyApplicationURLsForBundleIdentifier(bundle_id, &mut out);

            if !out.is_null() {
                crate::warning(&format!(
                    "warning: error searching for '{:?}' application instances",
                    app_type
                ));
                continue;
            }

            let count: isize = CFArrayGetCount(app_urls);

            for index in 0..count {
                let url: CFURLRef = CFArrayGetValueAtIndex(app_urls, index) as CFURLRef;
                if url.is_null() {
                    // This shouldn't happen, so ignore it.
                    crate::warning("CFURLRef was unexpectedly NULL");
                    continue;
                }

                match get_app_from_url(url, Some(app_type.clone())) {
                    Ok(app) => app_bundles.push(app),
                    Err(err) => {
                        // TODO: Do something else here?
                        //       We don't want this to be a catastrophic error,
                        //       because one "corrupted" app installation shouldn't
                        //       prevent us from returning a list of other valid
                        //       installations. But we should inform the user of this
                        //       somehow.
                        crate::warning(&format!(
                            "warning: wolfram app had unexpected or invalid\
                            structure: {}",
                            err
                        ))
                    },
                }
            }

            CFRelease(app_urls as *const _);
            CFRelease(bundle_id as *const _);
        }
    }

    app_bundles
}
