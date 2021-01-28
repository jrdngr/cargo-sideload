use std::{collections::HashSet, fs::canonicalize, path::Path};

use cargo::{
    core::{
        resolver::EncodableResolve, Dependency, PackageId, Resolve, Source, SourceId, Summary,
        Workspace,
    },
    sources::RegistrySource,
    Config as CargoConfig,
};
use url::Url;

use crate::args::CargoSideloadCommonArgs;

pub fn create_registry<'cfg>(
    config: &'cfg CargoConfig,
    registry_name: &str,
) -> anyhow::Result<RegistrySource<'cfg>> {
    let index_url = registry_index_url(&config, registry_name)?;
    let url = Url::parse(&index_url)?;

    let source_id = SourceId::for_registry(&url)?;
    let yanked_whitelist = HashSet::new();

    Ok(RegistrySource::remote(
        source_id,
        &yanked_whitelist,
        &config,
    ))
}

/// Updates the local copy of a registry index
pub fn update_index<S: Source>(config: &CargoConfig, source: &mut S) -> anyhow::Result<()> {
    let _package_cache_lock = config.acquire_package_cache_lock()?;
    source.update()
}

pub fn package_summaries<S: Source>(
    config: &CargoConfig,
    source: &mut S,
    package: &str,
) -> anyhow::Result<Vec<Summary>> {
    let _package_cache_lock = config.acquire_package_cache_lock()?;

    let mut summaries = Vec::new();

    let dep = Dependency::new_override(package.into(), source.source_id());
    source.query(&dep, &mut |summary| summaries.push(summary))?;

    summaries.sort_by(|s1, s2| s1.version().cmp(s2.version()));

    Ok(summaries)
}

pub fn latest_version(summaries: &[Summary]) -> Option<&Summary> {
    summaries.iter().max_by_key(|summary| summary.version())
}

pub fn workspace_packages<'cfg>(
    config: &CargoConfig,
    args: &CargoSideloadCommonArgs,
    workspace: &Workspace<'cfg>,
) -> anyhow::Result<Vec<PackageId>> {
    let lock_file_path = args.path.join("Cargo.lock");
    if !lock_file_path.exists() {
        cargo::ops::generate_lockfile(workspace)?;
    }

    let lock_file_path = canonicalize(lock_file_path)?;
    let lock_file = parse_lockfile(&lock_file_path, &workspace)?;

    let registry_index_url = registry_index_url(config, &args.registry)?;

    let mut packages = Vec::new();

    for package_id in lock_file.iter() {
        let name = package_id.name().to_string();
        if let Some(packages) = &args.packages {
            if !packages.contains(&name) {
                continue;
            }
        }

        let url = package_id.source_id().url().to_string();
        if url == registry_index_url {
            packages.push(package_id);
        }
    }

    Ok(packages)
}

/// Returns the name of the registry's directory in the local cache.
/// The result is in the format `[registry_name]-[hash]`
/// This function is copy/pasted from a private function in Cargo.
pub fn registry_directory(id: SourceId) -> String {
    let hash = cargo::util::hex::short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}

fn registry_index_url(config: &CargoConfig, registry_name: &str) -> anyhow::Result<String> {
    let registry_config = cargo::ops::registry_configuration(config, Some(registry_name.into()))?;

    match registry_config.index {
        Some(index) => Ok(index),
        None => anyhow::bail!(
            "No index available for registry named \"{}\"",
            registry_name
        ),
    }
}

fn parse_lockfile<'cfg, P: AsRef<Path>>(
    path: P,
    workspace: &Workspace<'cfg>,
) -> anyhow::Result<Resolve> {
    let toml_string = std::fs::read_to_string(path.as_ref())?;
    let toml: toml::Value =
        cargo::util::toml::parse(&toml_string, path.as_ref(), workspace.config())?;

    let encodable_resolve: EncodableResolve = toml.try_into()?;
    Ok(encodable_resolve.into_resolve(&toml_string, workspace)?)
}
