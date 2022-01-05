use clap::Parser;

use wolfram_app_discovery::{self as wad, WolframApp};

/// Discovery local installations of the Wolfram Language and Wolfram products.
#[derive(Parser, Debug)]
struct Args {}

fn main() -> Result<(), wad::Error> {
    let args = Args::parse();

    let app = WolframApp::try_default()?;

    println!("{:#?}", app.kernel_executable_path()?);
    println!("{:#?}", args);

    Ok(())
}
