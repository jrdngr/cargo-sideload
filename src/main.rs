pub mod download;
mod opts;
pub mod utils;

use std::fs::canonicalize;

use cargo::{core::Workspace, Config};
use clap::Clap;

fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    let opts = opts::Opts::parse();
    let config = Config::default()?;


    let mut downloader = download::Downloader::new(&config, &opts)?;

    let manifest_path = canonicalize(opts.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &config)?;

    let lock_file_path = canonicalize(opts.path.join("Cargo.lock"))?;
    let lock_file = utils::parse_lockfile(&lock_file_path, &workspace)?;
    
    let registry_index_url = utils::registry_index_url(&config, &opts.registry)?;

    for package_id in lock_file.iter() {
        let name = package_id.name().to_string();
        if let Some(packages) = &opts.packages {
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
