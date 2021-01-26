use std::fs::canonicalize;

use cargo::{core::Workspace, util::config::Config as CargoConfig};

use crate::{args::CargoSideloadOutdatedArgs, package_entry::PackageEntry, utils};

pub fn outdated(args: CargoSideloadOutdatedArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let manifest_path = canonicalize(args.common.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let mut registry = utils::create_registry(&cargo_config, &args.common.registry)?;

    let packages = utils::list_registry_packages(&cargo_config, &args.common, &workspace)?;
    let package_names: Vec<String> = packages
        .iter()
        .map(|package_id| package_id.name().to_string())
        .collect();

    utils::update_index_packages(&cargo_config, &mut registry, &package_names)?;

    let mut has_outdated_packages = false;

    for package_id in packages {
        let entries = PackageEntry::from_name(&cargo_config, &registry, &package_id.name())?;
        match entries.iter().max() {
            Some(latest) => {
                // Got a weird comparison error here
                // TODO Fix server::Version mismatch
                let latest_version = latest.version.to_string();
                let current_version = package_id.version().to_string();

                if current_version != latest_version {
                    has_outdated_packages = true;
                    println!(
                        "{} {} -> {}",
                        package_id.name(),
                        package_id.version(),
                        latest.version
                    );    
                }
            }
            None => println!("Package not found"),
        }
    }

    if args.error && has_outdated_packages {
        anyhow::bail!("Found outdated packages");
    }

    Ok(())
}
