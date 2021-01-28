use clap::Clap;
use std::path::PathBuf;

use crate::config::{Config, Header};

#[derive(Clap, Debug, Clone)]
#[clap(about, version)]
pub enum CargoSideloadArgs {
    /// Downloads all packages in your `Cargo.toml` and places them in the local Cargo cache, limited to the specified registry.
    Fetch(CargoSideloadFetchArgs),
    /// List some info for all published versions of the specified crate. Does not include yanked versions.
    List(CargoSideloadListArgs),
    /// List all crates in your `Cargo.toml` that have newer versions available, limited to the specified registry.
    Outdated(CargoSideloadOutdatedArgs),
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
    /// List of crates to run this command on.
    pub packages: Option<Vec<String>>,
    #[clap(short, long)]
    /// Silence Cargo
    pub quiet: bool,
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadFetchArgs {
    #[clap(flatten)]
    pub common: CargoSideloadCommonArgs,
    #[clap(long)]
    /// Headers to add to the download request in the format `[Header-Name]: [Header Value]`.  
    /// Example: `Authorization: Bearer abcdefg12345`  
    pub headers: Vec<Header>,
    #[clap(short, long)]
    /// Deletes any existing `.crate` file before downloading its replacement.
    pub force: bool,
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadListArgs {
    /// Name of the crate whose info will be returned
    pub name: String,
    #[clap(short, long, env = "CARGO_SIDELOAD_DEFAULT_REGISTRY")]
    /// Name of the registry as it is defined in your cargo config (usually `~/.cargo/config.toml`).
    pub registry: String,
    #[clap(short, long)]
    /// Only return info for the latest version of the package
    pub latest: bool,
    #[clap(short, long = "version-only")]
    /// Only return version numbers
    pub version_only: bool,
    #[clap(short, long)]
    /// Silence Cargo
    pub quiet: bool,
}

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadOutdatedArgs {
    #[clap(flatten)]
    pub common: CargoSideloadCommonArgs,
    #[clap(short, long)]
    /// Returns an error if any dependencies are out of date
    pub error: bool,
}

impl CargoSideloadArgs {
    pub fn load(config: &Config) -> Self {
        // Set the default registry from the user's config file before parsing the arguments
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

        // Add headers from the user's config file to the arg headers
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
