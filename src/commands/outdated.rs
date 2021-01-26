use std::fs::canonicalize;

use cargo::{
    core::{source::Source, Workspace},
    util::config::Config as CargoConfig,
};

use crate::{args::CargoSideloadOutdatedArgs, utils};

pub fn outdated(args: CargoSideloadOutdatedArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let manifest_path = canonicalize(args.common.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let mut registry = utils::create_registry(&cargo_config, &args.common.registry)?;
    let packages = utils::workspace_packages(&cargo_config, &args.common, &workspace)?;

    utils::update_index(&cargo_config, &mut registry)?;

    let mut has_outdated_packages = false;

    let _package_cache_lock = cargo_config.acquire_package_cache_lock()?;

    for package_id in packages {
        if registry.is_yanked(package_id)? {
            println!("{} {} -> yanked", package_id.name(), package_id.version());
            has_outdated_packages = true;
            continue;
        }

        let summaries = utils::package_summaries(&cargo_config, &mut registry, &package_id.name())?;
        let latest_version_summary = utils::latest_version(&summaries);

        match latest_version_summary {
            Some(latest) => {
                if package_id.version() < latest.version() {
                    has_outdated_packages = true;
                    println!(
                        "{} {} -> {}",
                        package_id.name(),
                        package_id.version(),
                        latest.version()
                    );
                }
            }
            None => println!("Package {} not found", package_id.name()),
        }
    }

    if args.error && has_outdated_packages {
        anyhow::bail!("Found outdated packages");
    }

    Ok(())
}
