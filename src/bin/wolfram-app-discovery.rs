use clap::Parser;

use wolfram_app_discovery::{self as wad, WolframApp, WolframProduct};

/// Discovery local installations of the Wolfram Language and Wolfram products.
#[derive(Parser, Debug)]
struct Args {
    /// Wolfram products to include.
    #[clap(long, arg_enum)]
    product: Vec<WolframProduct>,
}

fn main() -> Result<(), wad::Error> {
    let args: Args = Args::parse();

    let app = WolframApp::try_default()?;
    let wl_version = app.wolfram_version()?;

    println!("\nDefault Wolfram Language installation:\n");

    println!("  Product:                     {:?}", app.product());
    println!("  Wolfram Language version:    {}", wl_version);
    #[rustfmt::skip]
    println!("  $InstallationDirectory:      {}", app.installation_directory().display());

    Ok(())
}
