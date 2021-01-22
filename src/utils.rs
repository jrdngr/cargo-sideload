use std::path::Path;

use cargo::{
    core::{resolver::EncodableResolve, Resolve, SourceId, Workspace},
    ops::registry_configuration,
    Config as CargoConfig,
};

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

// This function is copy/pasted from a private function in `cargo`
pub fn registry_name(id: SourceId) -> String {
    let hash = cargo::util::hex::short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}
