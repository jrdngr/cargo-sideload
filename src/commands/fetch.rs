use std::fs::canonicalize;

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
        let registry = utils::create_registry(&config, &args.common.registry)?;
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
            self.delete_existing(package_id)?;
        }

        let maybe_package = self.download_crate_file(package_id)?;

        match maybe_package {
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

                // The code to unpack the crate is private, but we can trigger it by calling `Source::download` again.
                // This will see that the cached file is already present and attempt to unpack it.
                self.download_crate_file(package_id)?;
            }
        }

        Ok(())
    }

    fn download_crate_file(&mut self, package_id: PackageId) -> anyhow::Result<MaybePackage> {
        let _package_cache_lock = self.config.acquire_package_cache_lock()?;
        let result = self.registry.download(package_id);

        if result.is_err() {
            println!(
                "Failed to unpack crate file for {}. Double check your download url and headers.",
                package_id.name()
            );
            self.delete_existing(package_id)?;
        }

        result
    }

    /// Package cache path for the specified registry
    fn target_dir(&self, source_id: SourceId) -> Filesystem {
        let registry_directory = utils::registry_directory(source_id);
        self.config.registry_cache_path().join(&registry_directory)
    }

    /// Deletes an existing cached package file
    fn delete_existing(&self, package_id: PackageId) -> anyhow::Result<()> {
        let name = package_id.name();
        let version = package_id.version().to_string();

        let file_name = format!("{}-{}.crate", name, version);
        let file_lock = self.target_dir(self.registry.source_id()).open_rw(
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
