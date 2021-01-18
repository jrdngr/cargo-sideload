mod args;
pub mod download;
pub mod utils;

use std::fs::canonicalize;

use cargo::{core::Workspace, Config};
use clap::Clap;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Running `cargo sideload` will pass "sideload" as the first argument.
    // Since this isn't a real argument in the definition, the command will fail.
    let args = std::env::args_os().filter(|arg| arg != "sideload");

    let args = args::CargoSideloadArgs::parse_from(args);
    let config = Config::default()?;

    let mut downloader = download::Downloader::new(&config, &args)?;

    let manifest_path = canonicalize(args.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &config)?;

    let lock_file_path = canonicalize(args.path.join("Cargo.lock"))?;
    let lock_file = utils::parse_lockfile(&lock_file_path, &workspace)?;

    let registry_index_url = utils::registry_index_url(&config, &args.registry)?;

    for package_id in lock_file.iter() {
        let name = package_id.name().to_string();
        if let Some(packages) = &args.packages {
            if !packages.contains(&name) {
                continue;
            }
        }

        let url = package_id.source_id().url().to_string();
        if url == registry_index_url {
            let version = package_id.version().to_string();
            downloader.download(&name, &version)?;
        }
    }

    Ok(())
}
