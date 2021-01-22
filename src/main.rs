mod args;
pub mod commands;
pub mod config;
pub mod utils;

use crate::{args::CargoSideloadArgs, config::Config};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::load()?.unwrap_or_default();
    let args = CargoSideloadArgs::load(&config);

    if let Some(_subcommand) = args.subcommand {
        // Do something
    } else {
        commands::download(args)?;
    }

    Ok(())
}
