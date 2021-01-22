mod args;
pub mod commands;
pub mod config;
pub mod utils;

use crate::{
    args::{CargoSideloadArgs, CargoSideloadSubcommand},
    config::Config,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let config = Config::load()?.unwrap_or_default();
    let mut args = CargoSideloadArgs::load(&config);

    if let Some(subcommand) = args.subcommand.take() {
        match subcommand {
            CargoSideloadSubcommand::List(list_args) => commands::list(args, list_args)?,
        }
    } else {
        commands::download(args)?;
    }

    Ok(())
}
