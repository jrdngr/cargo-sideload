use cargo::{
    core::{
        package_id::PackageId,
        source::{MaybePackage, Source, SourceId},
    },
    sources::registry::RegistrySource,
    util::{config::Config, Filesystem},
};
use std::collections::HashSet;
use url::Url;

use crate::{
    args::{AuthHeader, CargoSideloadArgs},
    utils::registry_index_url,
};

pub struct Downloader<'cfg> {
    config: &'cfg Config,
    registry: RegistrySource<'cfg>,
    client: reqwest::blocking::Client,
    auth_header: Option<AuthHeader>,
}

impl<'cfg> Downloader<'cfg> {
    pub fn new(config: &'cfg Config, args: &CargoSideloadArgs) -> anyhow::Result<Self> {
        let index_url = registry_index_url(&config, &args.registry)?;
        let url = Url::parse(&index_url)?;

        let source_id = SourceId::for_registry(&url)?;

        let yanked_whitelist = HashSet::new();
        let registry = RegistrySource::remote(source_id, &yanked_whitelist, config);
        let client = reqwest::blocking::Client::new();

        Ok(Self {
            config,
            registry,
            client,
            auth_header: args.auth_header.clone(),
        })
    }

    pub fn download(&mut self, name: &str, version: &str) -> anyhow::Result<()> {
        let source_id = self.registry.source_id();
        let package_id = PackageId::new(name, version, source_id)?;

        let result = {
            let _package_cache_lock = self.config.acquire_package_cache_lock()?;
            self.registry.download(package_id)?
        };

        match result {
            MaybePackage::Ready(_) => println!("{}-{} is already cached.", name, version),
            MaybePackage::Download { url, .. } => {
                println!("Downloading: {}", url);

                let mut request_builder = self.client.get(&url);

                if let Some(ref auth_header) = self.auth_header {
                    request_builder = request_builder.header(&auth_header.name, &auth_header.value);
                }

                let body = request_builder.send()?.bytes()?;

                let file_name = format!("{}-{}.crate", name, version);

                {
                    let file_lock = target_dir(source_id, &self.config).open_rw(
                        file_name,
                        &self.config,
                        "Waiting for file lock...",
                    )?;

                    let file_path = file_lock.path();

                    std::fs::write(file_path, body)?;
                    println!("Downloaded: {:?}", file_path);
                }
            }
        }

        Ok(())
    }
}

fn target_dir(source_id: SourceId, config: &Config) -> Filesystem {
    let registry_name = registry_name(source_id);
    config.registry_cache_path().join(&registry_name)
}

// This function is copy/pasted from a private function in `cargo`
fn registry_name(id: SourceId) -> String {
    let hash = cargo::util::hex::short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}
