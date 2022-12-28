mod output;


use std::path::PathBuf;

use clap::Parser;

use wolfram_app_discovery::{self as wad, Filter, WolframApp, WolframAppType};

use self::output::{Property, PropertyValue};

/// Find local installations of the Wolfram Language and Wolfram apps.
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
    Default {
        #[clap(flatten)]
        discovery: DiscoveryOpts,

        #[clap(flatten)]
        output: SingleOutputOpts,
    },
    /// List all locatable Wolfram apps.
    #[clap(display_order(2))]
    List {
        #[clap(flatten)]
        discovery: DiscoveryOpts,

        #[clap(flatten)]
        output: OutputOpts,
    },
    /// Print information about a specified Wolfram application.
    #[clap(display_order(3))]
    Inspect {
        app_dir: PathBuf,

        #[clap(flatten)]
        opts: SingleOutputOpts,

        #[clap(flatten)]
        debug: Debug,
    },
    // For generating `docs/CommandLineHelp.md`.
    #[clap(hide = true)]
    PrintAllHelp {
        #[arg(long, required = true)]
        markdown: bool,
    },
}

//======================================
// Arguments and options parsing
//======================================

/// CLI arguments that affect which apps get discovered.
#[derive(Debug, Clone)]
#[derive(Parser)]
struct DiscoveryOpts {
    /// Wolfram application types to include.
    #[arg(long = "app-type", value_enum)]
    app_types: Vec<WolframAppType>,

    #[clap(flatten)]
    debug: Debug,
}

/// CLI arguments used by commands that work on a single app instance (i.e. `default`
/// and `inspect`, but not `list`).
#[derive(Debug, Clone)]
#[derive(Parser)]
struct SingleOutputOpts {
    /// If specified, the value of this property will be written without any
    /// trailing newline.
    ///
    /// This is useful when using `wolfram-app-discovery` to initialize the
    /// value of variables in shell scripts or build scripts (e.g. CMake).
    #[arg(long, value_name = "PROPERTY", conflicts_with_all = ["format", "properties", "all_properties"])]
    raw_value: Option<Property>,

    #[clap(flatten)]
    output_opts: OutputOpts,
}

/// CLI arguments that affect the content and format of the output.
#[derive(Debug, Clone)]
#[derive(Parser)]
struct OutputOpts {
    /// Properties to output.
    #[arg(
        long = "property",
        alias = "properties",
        value_enum,
        // Allow `--properties=prop1,prop2,etc`
        value_delimiter = ',',
        default_values = ["app-type", "wolfram-version", "app-directory"]
    )]
    properties: Vec<Property>,

    /// If set, all available properties will be printed.
    #[arg(long, conflicts_with = "properties")]
    all_properties: bool,

    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

/// The format to use when writing output.
#[derive(Debug, Clone)]
#[derive(clap::ValueEnum)]
enum OutputFormat {
    Text,
    CSV,
}

#[derive(Debug, Clone)]
#[derive(Parser)]
struct Debug {
    /// Whether to print application information in the verbose Debug format.
    #[arg(long)]
    debug: bool,
}

//======================================
// main()
//======================================

fn main() -> Result<(), wad::Error> {
    let Args { command } = Args::parse();

    match command {
        Command::Default { discovery, output } => default(discovery, output),
        Command::List { discovery, output } => list(discovery, output),
        Command::Inspect {
            app_dir,
            opts,
            debug,
        } => inspect(app_dir, &opts, debug),
        Command::PrintAllHelp { markdown } => {
            // This is a required argument for the time being.
            assert!(markdown);

            let () = clap_markdown::print_help_markdown::<Args>();

            Ok(())
        },
    }
}

//======================================
// Subcommand entrypoints
//======================================

fn default(
    discovery: DiscoveryOpts,
    single_output: SingleOutputOpts,
) -> Result<(), wad::Error> {
    let DiscoveryOpts { app_types, debug } = discovery;

    let filter = make_filter(app_types);

    let app = WolframApp::try_default_with_filter(&filter)?;

    print_single_app(&app, &single_output, debug)?;

    Ok(())
}

fn list(discovery: DiscoveryOpts, output: OutputOpts) -> Result<(), wad::Error> {
    let DiscoveryOpts { app_types, debug } = discovery;

    let filter = make_filter(app_types);

    let OutputOpts {
        format,
        properties,
        all_properties,
    } = &output;

    let apps: Vec<WolframApp> = wad::discover_with_filter(&filter);

    let properties: &[Property] = match all_properties {
        true => Property::variants(),
        false => properties,
    };

    match format {
        OutputFormat::Text => {
            for (index, app) in apps.iter().enumerate() {
                println!("\nWolfram App #{}:\n", index);
                print_app_info(app, &output, debug.debug)?;
            }
        },
        OutputFormat::CSV => {
            let mut stdout = std::io::stdout();

            output::write_csv_header(&mut stdout, properties)
                .expect("error formatting CSV header");

            for app in &apps {
                output::write_csv_row(&mut stdout, app, properties)
                    .expect("error formatting CSV row");
            }
        },
    }


    Ok(())
}

fn inspect(
    location: PathBuf,
    opts: &SingleOutputOpts,
    debug: Debug,
) -> Result<(), wad::Error> {
    let app = WolframApp::from_app_directory(location)?;

    print_single_app(&app, opts, debug)
}

//======================================
// Utility functions
//======================================

fn print_single_app(
    app: &WolframApp,
    opts: &SingleOutputOpts,
    debug: Debug,
) -> Result<(), wad::Error> {
    let SingleOutputOpts {
        raw_value,
        output_opts,
    } = opts;

    if let Some(prop) = raw_value {
        // NOTE: Use print! instead of println! to avoid printing a newline,
        //       which would require the user to remove the newline in some
        //       use-cases.
        print!("{}", PropertyValue(&app, prop.clone()));

        return Ok(());
    }

    print_app_info(app, output_opts, debug.debug)
}

fn print_app_info(
    app: &WolframApp,
    opts: &OutputOpts,
    debug: bool,
) -> Result<(), wad::Error> {
    let OutputOpts {
        format,
        properties,
        all_properties,
    } = opts;

    if debug {
        println!("{:#?}", app);
        return Ok(());
    }

    let properties: &[Property] = match all_properties {
        true => Property::variants(),
        false => properties,
    };

    match format {
        OutputFormat::Text => {
            for prop in properties {
                let value = PropertyValue(app, prop.clone());

                let name = format!("{prop}:");

                println!("{name:<width$} {value}", width = 35);
            }
        },
        OutputFormat::CSV => {
            let mut stdout = std::io::stdout();

            output::write_csv_header(&mut stdout, properties)
                .expect("error formatting CSV header");
            output::write_csv_row(&mut stdout, app, properties)
                .expect("error formatting CSV row");
        },
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
