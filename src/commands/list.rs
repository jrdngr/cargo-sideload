use std::collections::HashSet;

use cargo::{
    core::source::{Source, SourceId},
    sources::registry::RegistrySource,
    util::config::Config as CargoConfig,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    args::{CargoSideloadArgs, CargoSideloadListArgs},
    utils,
};

pub fn list(args: CargoSideloadArgs, list_args: CargoSideloadListArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let index_url = utils::registry_index_url(&cargo_config, &args.registry)?;
    let url = Url::parse(&index_url)?;

    let source_id = SourceId::for_registry(&url)?;
    let yanked_whitelist = HashSet::new();

    let mut registry = RegistrySource::remote(source_id, &yanked_whitelist, &cargo_config);

    {
        let _package_cache_lock = cargo_config.acquire_package_cache_lock()?;
        registry.update()?;
    }

    let package_path = cargo_config
        .registry_index_path()
        .join(utils::registry_name(source_id))
        .join(".cache")
        .join(utils::package_dir(&list_args.name));

    let file_lock =
        package_path.open_ro(&list_args.name, &cargo_config, "Waiting for file lock...")?;

    let file_path = file_lock.path();
    let package_info = std::fs::read_to_string(file_path)?;
    let entries = create_package_entries(&package_info)?;

    dbg!(entries);

    Ok(())
}

fn create_package_entries(package_info: &str) -> anyhow::Result<Vec<PackageEntry>> {
    let package_entries = package_info
        .lines()
        .map(trim_package_line)
        .map(serde_json::from_str)
        .collect::<Result<Vec<PackageEntry>, _>>()?;

    Ok(package_entries)
}

// This isn't hacky at all
fn trim_package_line(line: &str) -> &str {
    let start_index = line.find('{').unwrap_or(0);
    let trimmed_line = line.split_at(start_index).1.trim_end_matches('\u{0}');
    trimmed_line
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageEntry {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: String,
    #[serde(rename = "cksum")]
    pub checksum: String,
    pub yanked: bool,
}
