use std::{path::PathBuf, str::FromStr};

use clap::Clap;

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
    pub headers: Vec<AuthHeader>,
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

#[derive(Debug, Clone)]
pub struct AuthHeader {
    pub name: String,
    pub value: String,
}

impl FromStr for AuthHeader {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.splitn(2, ':').collect();

        if split.len() != 2 {
            anyhow::bail!("Invalid auth header format. Expected `[Header-Name]: [Header Value]`");
        }

        let name = split[0].to_string();
        let value = split[1].trim_start().to_string();

        Ok(Self { name, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header() {
        let auth_header = AuthHeader::from_str("Authorization: Bearer abcd1234").unwrap();
        assert_eq!(auth_header.name, "Authorization");
        assert_eq!(auth_header.value, "Bearer abcd1234");

        assert!(AuthHeader::from_str("Authorization").is_err());
        assert!(AuthHeader::from_str("").is_err());
    }
}
