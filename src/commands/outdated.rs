use std::fs::canonicalize;

use cargo::{core::Workspace, util::config::Config as CargoConfig};

use crate::{args::CargoSideloadArgs, utils};

pub fn outdated(args: CargoSideloadArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let manifest_path = canonicalize(args.path.join("Cargo.toml"))?;
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let mut registry = utils::create_registry(&cargo_config, &args)?;

    let packages = utils::list_registry_packages(&cargo_config, &args, &workspace)?;
    let package_names: Vec<String> = packages
        .iter()
        .map(|package_id| package_id.name().to_string())
        .collect();

    utils::update_index_packages(&cargo_config, &mut registry, &package_names)?;

    for _package_id in packages {
        // Check package
    }

    Ok(())
}
