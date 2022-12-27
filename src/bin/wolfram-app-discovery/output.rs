use std::{
    fmt::{self, Display},
    io,
};

use wolfram_app_discovery::WolframApp;

/// A property of a Wolfram installation that can be discovered.
#[derive(Debug, Clone, PartialEq)]
#[derive(clap::ValueEnum)]
pub enum Property {
    /// [`WolframAppType`] value describing the installation.
    ///
    /// [`WolframAppType`]: https://docs.rs/wolfram-app-discovery/latest/wolfram_app_discovery/enum.WolframAppType.html
    AppType,

    AppDirectory,

    /// [`WolframVersion`] value of the installation.
    ///
    /// [`WolframVersion`]: https://docs.rs/wolfram-app-discovery/latest/wolfram_app_discovery/struct.WolframVersion.html
    WolframVersion,

    /// [`$InstallationDirectory`] value of the installation.
    ///
    /// [`$InstallationDirectory`]: https://reference.wolfram.com/language/ref/$InstallationDirectory
    InstallationDirectory,

    /// Wolfram *LibraryLink* C includes directory
    LibraryLinkCIncludesDirectory,
}

/// Represents the value of the specified property on the given app for the
/// purposes of formatting.
///
/// The purpose of this type is to implement [`Display`].
///
pub struct PropertyValue<'app>(pub &'app WolframApp, pub Property);

//==========================================================
// Impls
//==========================================================

impl Property {
    pub const fn variants() -> &'static [Property] {
        // NOTE: Whenever the match statement below causes a compile time failure
        //       because a variant has been added, update the returned slice to
        //       include the new variant.
        if false {
            #[allow(unused_variables)]
            let property: Property = unreachable!();

            #[allow(unreachable_code)]
            match property {
                Property::AppType
                | Property::WolframVersion
                | Property::AppDirectory
                | Property::InstallationDirectory
                | Property::LibraryLinkCIncludesDirectory => unreachable!(),
            }
        }

        &[
            Property::AppType,
            Property::WolframVersion,
            Property::AppDirectory,
            Property::InstallationDirectory,
            Property::LibraryLinkCIncludesDirectory,
        ]
    }
}

//==========================================================
// CSV
//==========================================================

pub fn write_csv_header(
    fmt: &mut dyn io::Write,
    properties: &[Property],
) -> io::Result<()> {
    let header: String = properties
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(",");

    writeln!(fmt, "{header}")
}

pub fn write_csv_row(
    fmt: &mut dyn io::Write,
    app: &WolframApp,
    properties: &[Property],
) -> io::Result<()> {
    for (index, prop) in properties.iter().cloned().enumerate() {
        let value = format!("{}", PropertyValue(app, prop));

        // Write the value as an escaped string.
        // TODO: Find a better method for CSV-escaping values.
        write!(fmt, "{value:?}")?;

        // If this isn't the last column, write a comma separator.
        if index != properties.len() - 1 {
            write!(fmt, ",")?;
        }
    }

    write!(fmt, "\n")
}

//======================================
// Display and formatting
//======================================

impl Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Property::AppType => "App type",
            Property::WolframVersion => "Wolfram Language version",
            Property::AppDirectory => "Application directory",
            Property::InstallationDirectory => "$InstallationDirectory",
            Property::LibraryLinkCIncludesDirectory => "LibraryLink C includes directory",
        };

        write!(f, "{name}")
    }
}

impl<'app> Display for PropertyValue<'app> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PropertyValue(app, property) = self;

        match property {
            Property::AppType => {
                write!(fmt, "{:?}", app.app_type())
            },
            Property::WolframVersion => match app.wolfram_version() {
                Ok(version) => write!(fmt, "{version}"),
                Err(error) => {
                    // Print an error to stderr.
                    eprintln!("Error getting WolframVersion value: {error}");

                    write!(fmt, "Error")
                },
            },
            Property::AppDirectory => {
                write!(fmt, "{}", app.app_directory().display())
            },
            Property::InstallationDirectory => {
                write!(fmt, "{}", app.installation_directory().display())
            },
            Property::LibraryLinkCIncludesDirectory => match app
                .library_link_c_includes_directory()
            {
                Ok(value) => write!(fmt, "{}", value.display()),
                Err(error) => {
                    // Print an error to stderr.
                    eprintln!("Error getting LibraryLink C includes directory: {error}");

                    write!(fmt, "Error")
                },
            },
        }
    }
}
