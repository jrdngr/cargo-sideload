use std::{collections::HashSet, fs::canonicalize, path::Path};

use cargo::{
    core::{
        resolver::EncodableResolve, Dependency, PackageId, Resolve, Source, SourceId, Workspace,
    },
    ops::registry_configuration,
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

pub fn registry_index_url(config: &CargoConfig, registry_name: &str) -> anyhow::Result<String> {
    match registry_configuration(config, Some(registry_name.into()))?.index {
        Some(index) => Ok(index),
        None => anyhow::bail!(
            "No index available for registry named \"{}\"",
            registry_name
        ),
    }
}

pub fn parse_lockfile<'cfg, P: AsRef<Path>>(
    path: P,
    workspace: &Workspace<'cfg>,
) -> anyhow::Result<Resolve> {
    let toml_string = std::fs::read_to_string(path.as_ref())?;
    let toml: toml::Value =
        cargo::util::toml::parse(&toml_string, path.as_ref(), workspace.config())?;

    let encodable_resolve: EncodableResolve = toml.try_into()?;
    Ok(encodable_resolve.into_resolve(&toml_string, workspace)?)
}

pub fn update_index_packages<S: Source>(
    config: &CargoConfig,
    source: &mut S,
    packages: &[String],
) -> anyhow::Result<()> {
    let _package_cache_lock = config.acquire_package_cache_lock()?;
    // Fetch the updated index repo
    source.update()?;

    // Query the dependencies to create/update the file we're going to read below
    for package in packages {
        let dep = Dependency::new_override(package.into(), source.source_id());
        source.query(&dep, &mut |_| {})?;
    }

    Ok(())
}

pub fn list_registry_packages<'cfg>(
    config: &CargoConfig,
    args: &CargoSideloadCommonArgs,
    workspace: &Workspace<'cfg>,
) -> anyhow::Result<Vec<PackageId>> {
    let lock_file_path = canonicalize(args.path.join("Cargo.lock"))?;
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

// This function is copy/pasted from a private function in Cargo.
pub fn registry_name(id: SourceId) -> String {
    let hash = cargo::util::hex::short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}

// This function is copy/pasted from a private function in Cargo.
pub fn package_dir(package_name: &str) -> String {
    match package_name.len() {
        1 => "1".to_string(),
        2 => "2".to_string(),
        3 => format!("3/{}", &package_name[..1]),
        _ => format!("{}/{}", &package_name[0..2], &package_name[2..4]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_dir() {
        assert_eq!("1", package_dir("m"));
        assert_eq!("2", package_dir("my"));
        assert_eq!("3/m", package_dir("my_"));
        assert_eq!("my/_l", package_dir("my_lib"));
    }
}
