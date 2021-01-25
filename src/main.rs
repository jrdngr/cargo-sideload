mod args;
pub mod commands;
pub mod config;
pub mod package_entry;
pub mod utils;

use crate::{
    args::CargoSideloadArgs,
    config::Config,
};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    dotenv::dotenv().ok();

    let config = Config::load()?.unwrap_or_default();
    let args = CargoSideloadArgs::load(&config);

    match args{
        CargoSideloadArgs::Fetch(fetch_args) => commands::fetch(fetch_args)?,
        CargoSideloadArgs::List(list_args) => commands::list(list_args)?,
        CargoSideloadArgs::Outdated(od_args) => commands::outdated(od_args)?,
    }

    Ok(())
}
