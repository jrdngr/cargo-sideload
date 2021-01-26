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
use log::debug;
use url::Url;

use crate::{args::CargoSideloadFetchArgs, utils};

pub fn fetch(args: CargoSideloadFetchArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let manifest_path = canonicalize(args.common.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let mut downloader = Downloader::new(&cargo_config, &args)?;

    for package_id in utils::workspace_packages(&cargo_config, &args.common, &workspace)? {
        downloader.download(&package_id.name(), &package_id.version().to_string())?;
    }

    Ok(())
}

/// Downloads packages from a particular registry
struct Downloader<'cfg> {
    config: &'cfg CargoConfig,
    registry: RegistrySource<'cfg>,
    client: reqwest::blocking::Client,
    args: CargoSideloadFetchArgs,
}

impl<'cfg> Downloader<'cfg> {
    pub fn new(config: &'cfg CargoConfig, args: &CargoSideloadFetchArgs) -> anyhow::Result<Self> {
        let index_url = utils::registry_index_url(&config, &args.common.registry)?;
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

    /// Download the specified version of a package.
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
                debug!("GET {}", url);

                for header in &self.args.headers {
                    request_builder = request_builder.header(&header.name, &header.value);
                    debug!("HEADER {}: {}", header.name, header.value);
                }

                let response = request_builder.send()?;
                debug!("{:#?}", response);

                let body = response.error_for_status()?.bytes()?;
                debug!("BODY");
                debug!("{}", String::from_utf8_lossy(&body));
                debug!("END");

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

    /// Package cache path for the specified registry
    fn target_dir(&self, source_id: SourceId) -> Filesystem {
        let registry_directory = utils::registry_directory(source_id);
        self.config.registry_cache_path().join(&registry_directory)
    }

    /// Deletes an existing cached package file
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
