use cargo::{core::source::Source, util::config::Config as CargoConfig};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: Version,
    #[serde(rename = "cksum")]
    pub checksum: String,
    pub yanked: bool,
}

impl PackageEntry {
    pub fn from_name<S: Source>(
        config: &CargoConfig,
        source: &S,
        name: &str,
    ) -> anyhow::Result<Vec<Self>> {
        let package_path = config
            .registry_index_path()
            .join(utils::registry_name(source.source_id()))
            .join(".cache")
            .join(utils::package_dir(&name));

        let file_lock = package_path.open_ro(&name, &config, "Waiting for file lock...")?;

        let file_path = file_lock.path();
        if !file_path.exists() {
            anyhow::bail!("No package found with name {}", name);
        }

        let package_info = std::fs::read_to_string(file_path)?;
        Ok(create_package_entries(&package_info))
    }
}

impl std::cmp::PartialOrd for PackageEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

impl std::cmp::Ord for PackageEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

// We get some useful lines along with some filler lines by splitting on '\u{0}'.
// We'll just discard the filler lines and return whatever successfully parses.
fn create_package_entries(package_info: &str) -> Vec<PackageEntry> {
    package_info
        .split('\u{0}')
        .map(serde_json::from_str)
        .flatten()
        .collect()
}
