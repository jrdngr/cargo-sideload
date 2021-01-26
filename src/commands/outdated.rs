use std::fs::canonicalize;

use cargo::{core::Workspace, util::config::Config as CargoConfig};

use crate::{args::CargoSideloadCommonArgs, package_entry::PackageEntry, utils};

pub fn outdated(args: CargoSideloadCommonArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let manifest_path = canonicalize(args.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let mut registry = utils::create_registry(&cargo_config, &args.registry)?;

    let packages = utils::list_registry_packages(&cargo_config, &args, &workspace)?;
    let package_names: Vec<String> = packages
        .iter()
        .map(|package_id| package_id.name().to_string())
        .collect();

    utils::update_index_packages(&cargo_config, &mut registry, &package_names)?;

    for package_id in packages {
        let entries = PackageEntry::from_name(&cargo_config, &registry, &package_id.name())?;
        match entries.iter().max() {
            Some(latest) => {
                // Got a weird comparison error here
                // TODO Fix server::Version mismatch
                let latest_version = latest.version.to_string();
                let current_version = package_id.version().to_string();

                if current_version != latest_version {
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

    Ok(())
}
