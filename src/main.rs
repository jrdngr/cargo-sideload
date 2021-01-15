use std::{collections::HashSet, path::PathBuf};
use cargo::{
    core::{
        package_id::PackageId,
        source::{Source, SourceId},
    },
    sources::registry::RegistrySource,
    util::config::Config,
};
use url::Url;

const INDEX_URL: &str = "https://github.com/picklenerd/my-index";

fn main() -> anyhow::Result<()> {
    let path = PathBuf::from(INDEX_URL);
    let url = Url::parse(INDEX_URL)?;
    let source_id = SourceId::for_registry(&url)?;
    let yanked_whitelist = HashSet::new();
    let config = Config::default()?;

    let mut registry = RegistrySource::remote(source_id, &yanked_whitelist, &config);

    let package_id = PackageId::new("my_lib", "0.1.0", source_id)?;

    {
        config.acquire_package_cache_lock()?;
        registry.download(package_id)?;
    }

    Ok(())
}
