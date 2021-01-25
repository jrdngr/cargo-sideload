use cargo::{core::source::Source, util::config::Config as CargoConfig};

use crate::{
    args::CargoSideloadListArgs,
    package_entry::PackageEntry,
    utils,
};

pub fn list(args: CargoSideloadListArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let mut registry = utils::create_registry(&cargo_config, &args.registry)?;

    utils::update_index_packages(&cargo_config, &mut registry, &[args.name.clone()])?;

    let package_path = cargo_config
        .registry_index_path()
        .join(utils::registry_name(registry.source_id()))
        .join(".cache")
        .join(utils::package_dir(&args.name));

    let file_lock =
        package_path.open_ro(&args.name, &cargo_config, "Waiting for file lock...")?;

    let file_path = file_lock.path();
    if !file_path.exists() {
        anyhow::bail!("No package found with name {}", args.name);
    }

    let entries = PackageEntry::from_name(&cargo_config, &registry, &args.name)?;

    if args.latest {
        print_latest(&entries);
    } else if args.yanked {
        print_yanked(&entries)?;
    } else {
        print_published(&entries)?;
    }

    Ok(())
}

fn print_published(entries: &[PackageEntry]) -> anyhow::Result<()> {
    for entry in entries {
        if !entry.yanked {
            println!("{}", serde_json::to_string_pretty(&entry)?);
        }
    }

    Ok(())
}

fn print_latest(entries: &[PackageEntry]) {
    match entries.iter().max() {
        Some(latest) => println!("{}", latest.version),
        None => println!("Package not found"),
    }
}

fn print_yanked(entries: &[PackageEntry]) -> anyhow::Result<()> {
    for entry in entries {
        if entry.yanked {
            println!("{}", serde_json::to_string_pretty(&entry)?);
        }
    }

    Ok(())
}
