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

    let mut entries = Vec::<PackageEntry>::new();
    for line in package_info.lines() {
        let start_index = line.find("{").unwrap_or(0);
        let trimmed_line = line.split_at(start_index);
        entries.push(serde_json::from_str(&trimmed_line.1)?);
    }

    dbg!(entries);

    Ok(())
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
