use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
pub struct Opts {
    #[clap(long = "registry", env = "CARGO_SIDELOAD_REGISTRY")]
    pub registry: String,
    #[clap(long = "path", default_value = ".")]
    pub path: PathBuf,
    #[clap(
        long = "access_token",
        env = "CARGO_SIDELOAD_ACCESS_TOKEN",
        hide_env_values = true
    )]
    pub access_token: Option<String>,
    #[clap(short = 'p', long = "packages")]
    pub packages: Option<Vec<String>>,
}
