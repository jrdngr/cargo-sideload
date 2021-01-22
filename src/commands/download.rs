use std::{collections::HashSet, fs::canonicalize};

use cargo::{
    core::{
        package_id::PackageId,
        source::{MaybePackage, Source, SourceId},
        Workspace,
    },
    sources::registry::RegistrySource,
    util::{config::Config as CargoConfig, Filesystem},
};
use url::Url;

use crate::{args::CargoSideloadArgs, utils};

pub fn download(args: CargoSideloadArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;

    let mut downloader = Downloader::new(&cargo_config, &args)?;

    let manifest_path = canonicalize(args.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let lock_file_path = canonicalize(args.path.join("Cargo.lock"))?;
    let lock_file = utils::parse_lockfile(&lock_file_path, &workspace)?;

    let registry_index_url = utils::registry_index_url(&cargo_config, &args.registry)?;

    for package_id in lock_file.iter() {
        let name = package_id.name().to_string();
        if let Some(packages) = &args.packages {
            if !packages.contains(&name) {
                continue;
            }
        }

        let url = package_id.source_id().url().to_string();
        if url == registry_index_url {
            let version = package_id.version().to_string();
            downloader.download(&name, &version)?;
        }
    }

    Ok(())
}

pub struct Downloader<'cfg> {
    config: &'cfg CargoConfig,
    registry: RegistrySource<'cfg>,
    client: reqwest::blocking::Client,
    args: CargoSideloadArgs,
}

impl<'cfg> Downloader<'cfg> {
    pub fn new(config: &'cfg CargoConfig, args: &CargoSideloadArgs) -> anyhow::Result<Self> {
        let index_url = utils::registry_index_url(&config, &args.registry)?;
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
                if self.args.debug {
                    println!("GET {}", url);
                }

                for header in &self.args.headers {
                    request_builder = request_builder.header(&header.name, &header.value);
                    if self.args.debug {
                        println!("HEADER {}: {}", header.name, header.value);
                    }
                }

                let response = request_builder.send()?;
                if self.args.debug {
                    println!("{:#?}", response);
                }

                let body = response.error_for_status()?.bytes()?;
                if self.args.debug {
                    println!("BODY");
                    println!("{}", String::from_utf8_lossy(&body));
                    println!("END");
                }

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
        let registry_name = utils::registry_name(source_id);
        self.config.registry_cache_path().join(&registry_name)
    }

    fn delete_existing(&self, source_id: SourceId, package_id: PackageId) -> anyhow::Result<()> {
        let name = package_id.name();
        let version = package_id.version().to_string();

        let file_name = format!("{}-{}.crate", name, version);
        let file_lock = self.target_dir(source_id).open_rw(
            file_name,
            &self.config,
            "Waiting for file lock...",
        )?;

        let file_path = file_lock.path();

        std::fs::remove_file(file_path)?;
        println!("Removed: {:?}", file_path);

        Ok(())
    }
}
