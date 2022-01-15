mod print_all_help;

use std::path::PathBuf;

use clap::Parser;

use wolfram_app_discovery::{self as wad, Filter, WolframApp, WolframAppType};

/// Discovery local installations of the Wolfram Language and Wolfram products.
#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Print the default Wolfram app.
    ///
    /// This method uses [`WolframApp::try_default()`] to locate the default app.
    #[clap(display_order(1))]
    Default(AppOpts),
    /// List all locatable Wolfram apps.
    #[clap(display_order(2))]
    List(AppOpts),
    /// Print information about a specified Wolfram application.
    #[clap(display_order(3))]
    Inspect {
        app_dir: PathBuf,

        #[clap(flatten)]
        debug: Debug,
    },
    // For generating `docs/CommandLineHelp.md`.
    #[clap(setting(clap::AppSettings::Hidden))]
    PrintAllHelp {
        #[clap(long)]
        markdown: bool,
    },
}

#[derive(Parser, Debug)]
struct AppOpts {
    /// Wolfram application types to include.
    #[clap(long = "app-type", arg_enum)]
    app_types: Vec<WolframAppType>,

    #[clap(flatten)]
    debug: Debug,
}

#[derive(Parser, Debug)]
struct Debug {
    /// Whether to print application information in the verbose Debug format.
    #[clap(long)]
    debug: bool,
}

//======================================
// main()
//======================================

fn main() -> Result<(), wad::Error> {
    let Args { command } = Args::parse();

    match command {
        Command::Default(opts) => default(opts),
        Command::List(opts) => list(opts),
        Command::Inspect { app_dir, debug } => inspect(app_dir, debug.debug),
        Command::PrintAllHelp { markdown } => {
            print_all_help::print_all_help(markdown);
            Ok(())
        },
    }
}

//======================================
// Subcommand entrypoints
//======================================

fn default(AppOpts { app_types, debug }: AppOpts) -> Result<(), wad::Error> {
    let filter = make_filter(app_types);

    let app = WolframApp::try_default_with_filter(&filter)?;

    println!("\nDefault Wolfram Language installation:\n");

    print_app_info(&app, debug.debug)?;

    Ok(())
}

fn list(AppOpts { app_types, debug }: AppOpts) -> Result<(), wad::Error> {
    let filter = make_filter(app_types);

    for (index, app) in wad::discover_with_filter(&filter).into_iter().enumerate() {
        println!("\nWolfram App #{}:\n", index);
        print_app_info(&app, debug.debug)?;
    }

    Ok(())
}

fn inspect(location: PathBuf, debug: bool) -> Result<(), wad::Error> {
    let app = WolframApp::from_app_directory(location)?;

    print_app_info(&app, debug)
}

//======================================
// Utility functions
//======================================

fn print_app_info(app: &WolframApp, debug: bool) -> Result<(), wad::Error> {
    if debug {
        println!("{:#?}", app);
    } else {
        let wl_version = app.wolfram_version()?;

        println!("  Product:                     {:?}", app.app_type());
        println!("  Wolfram Language version:    {}", wl_version);
        #[rustfmt::skip]
        println!("  Application directory:       {}", app.app_directory().display());
    }

    Ok(())
}

fn make_filter(app_types: Vec<WolframAppType>) -> Filter {
    let app_types = if app_types.is_empty() {
        None
    } else {
        Some(app_types)
    };

    Filter { app_types }
}
