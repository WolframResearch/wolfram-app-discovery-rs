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

    println!("{:#?}", app);
    println!("{:#?}", args);

    Ok(())
}
