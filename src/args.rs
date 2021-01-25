use clap::Clap;
use std::path::PathBuf;

use crate::config::{Config, Header};

#[derive(Clap, Debug, Clone)]
#[clap(about, version)]
pub enum CargoSideloadArgs {
    /// Downloads all packages from the specified registry and places them in the local Cargo cache.
    Fetch(CargoSideloadFetchArgs),
    /// List published version numbers for the specified crate. Does not include yanked versions.
    List(CargoSideloadListArgs),
    /// List all crates associated with this registry that have newer versions available.
    Outdated(CargoSideloadCommonArgs),
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadFetchArgs {
    #[clap(flatten)]
    pub common: CargoSideloadCommonArgs,
    #[clap(long = "headers")]
    /// Headers to add to the download request in the format `[Header-Name]: [Header Value]`.  
    /// Example: `Authorization: Bearer abcdefg12345`  
    pub headers: Vec<Header>,
    #[clap(long = "force")]
    /// Deletes any existing `.crate` file before downloading its replacement.
    pub force: bool,
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadListArgs {
    #[clap(
        short = 'r',
        long = "registry",
        env = "CARGO_SIDELOAD_DEFAULT_REGISTRY"
    )]
    /// Name of the registry as it is defined in your cargo config (usually `~/.cargo/config.toml`).
    pub registry: String,
    /// Name of the crate whose version numbers will be returned
    pub name: String,
    #[clap(long, conflicts_with = "yanked")]
    /// Only return the latest version number
    pub latest: bool,
    #[clap(long)]
    /// Returns all yanked version numbers
    pub yanked: bool,
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadCommonArgs {
    #[clap(
        short = 'r',
        long = "registry",
        env = "CARGO_SIDELOAD_DEFAULT_REGISTRY"
    )]
    /// Name of the registry as it is defined in your cargo config (usually `~/.cargo/config.toml`).
    pub registry: String,
    #[clap(long = "path", default_value = ".")]
    /// Path to the `Cargo.toml` file of the crate you're running this command on.
    pub path: PathBuf,
    #[clap(short = 'p', long = "packages")]
    /// Comma separated list of crates to download
    pub packages: Option<Vec<String>>,
}

impl CargoSideloadArgs {
    pub fn load(config: &Config) -> Self {
        if let Some(default_registry) = &config.default_registry {
            std::env::set_var("CARGO_SIDELOAD_DEFAULT_REGISTRY", default_registry);
        }

        // Running `cargo sideload` will pass "sideload" as the first argument.
        // Since this isn't a real argument in the Clap definition, parsing the args will fail.
        let args = std::env::args_os().enumerate().filter_map(|(index, arg)| {
            if index == 1 && arg == "sideload" {
                None
            } else {
                Some(arg)
            }
        });

        let mut result = Self::parse_from(args);

        if let CargoSideloadArgs::Fetch(ref mut fetch_args) = &mut result {
            if let Some(registry) = config.registries.get(&fetch_args.common.registry) {
                for header in &registry.headers {
                    fetch_args.headers.push(header.clone());
                }
            }
        }

        result
    }
}
