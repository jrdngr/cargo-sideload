use cargo::{
    core::{
        package_id::PackageId,
        source::{MaybePackage, Source, SourceId},
    },
    sources::registry::RegistrySource,
    util::{config::Config as CargoConfig, Filesystem},
};
use std::collections::HashSet;
use url::Url;

use crate::{args::CargoSideloadArgs, utils::registry_index_url};

pub struct Downloader<'cfg> {
    config: &'cfg CargoConfig,
    registry: RegistrySource<'cfg>,
    client: reqwest::blocking::Client,
    args: CargoSideloadArgs,
}

impl<'cfg> Downloader<'cfg> {
    pub fn new(config: &'cfg CargoConfig, args: &CargoSideloadArgs) -> anyhow::Result<Self> {
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
            args: args.clone(),
        })
    }

    pub fn download(&mut self, name: &str, version: &str) -> anyhow::Result<()> {
        let source_id = self.registry.source_id();
        let package_id = PackageId::new(name, version, source_id)?;

        if self.args.force {
            self.delete_existing(source_id, package_id)?;
        }

        let result = {
            let _package_cache_lock = self.config.acquire_package_cache_lock()?;
            self.registry.download(package_id)?
        };

        match result {
            MaybePackage::Ready(_) => println!("{}-{} is already cached.", name, version),
            MaybePackage::Download { url, .. } => {
                println!("Downloading: {}", url);                
                let mut request_builder = self.client.get(&url);
                self.print_debug(format!("GET {}", url));

                for header in &self.args.headers {
                    request_builder = request_builder.header(&header.name, &header.value);
                    self.print_debug(format!("HEADER {}: {}", header.name, header.value));
                }

                let body = request_builder.send()?.error_for_status()?.bytes()?;

                let file_name = format!("{}-{}.crate", name, version);

                {
                    let file_lock = self.target_dir(source_id).open_rw(
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

    fn target_dir(&self, source_id: SourceId) -> Filesystem {
        let registry_name = registry_name(source_id);
        self.config.registry_cache_path().join(&registry_name)
    }

    fn delete_existing(&self, source_id: SourceId, package_id: PackageId) -> anyhow::Result<()> {
        let name = package_id.name();
        let version = package_id.version().to_string();

        let file_name = format!("{}-{}.crate", name, version);

        {
            let file_lock = self.target_dir(source_id).open_rw(
                file_name,
                &self.config,
                "Waiting for file lock...",
            )?;

            let file_path = file_lock.path();

            std::fs::remove_file(file_path)?;
            println!("Removed: {:?}", file_path);
        }

        Ok(())
    }

    fn print_debug(&self, text: impl std::fmt::Display) {
        if self.args.debug {
            println!("{}", text);
        }
    }
}

// This function is copy/pasted from a private function in `cargo`
fn registry_name(id: SourceId) -> String {
    let hash = cargo::util::hex::short_hash(&id);
    let ident = id.url().host_str().unwrap_or("").to_string();
    format!("{}-{}", ident, hash)
}
