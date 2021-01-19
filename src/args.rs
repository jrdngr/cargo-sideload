use std::path::PathBuf;
use clap::Clap;

use crate::config::Header;

#[derive(Clap, Debug, Clone)]
pub struct CargoSideloadArgs {
    #[clap(short = 'r', long = "registry")]
    /// Name of the registry as it is defined in your cargo config (usually `~/.cargo/config.toml`).
    pub registry: String,
    #[clap(long = "path", default_value = ".")]
    /// Path to the `Cargo.toml` file of the crate you're running this command on.
    pub path: PathBuf,
    #[clap(short = 'p', long = "packages")]
    /// Comma separated list of crates to download
    pub packages: Option<Vec<String>>,
    #[clap(long = "headers")]
    /// Headers to add to the download request in the format `[Header-Name]: [Header Value]`.  
    /// Example: `Authorization: Bearer abcdefg12345`  
    pub headers: Vec<Header>,
    #[clap(long = "force")]
    /// Deletes any existing `.crate` file before downloading its replacement.
    pub force: bool,
}

impl CargoSideloadArgs {    
    pub fn load() -> Self {
        // Running `cargo sideload` will pass "sideload" as the first argument.
        // Since this isn't a real argument in the definition, the command will fail.
        let args = std::env::args_os()
            .enumerate()
            .filter_map(|(index, arg)| {
                if index == 1 && arg == "sideload" {
                    None
                } else {
                    Some(arg)
                }
            });

        Self::parse_from(args)
    }
}
