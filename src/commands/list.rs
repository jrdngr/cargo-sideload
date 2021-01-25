use cargo::{core::source::Source, util::config::Config as CargoConfig};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{
    args::{CargoSideloadArgs, CargoSideloadListArgs},
    utils,
};

pub fn list(args: CargoSideloadArgs, list_args: CargoSideloadListArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let mut registry = utils::create_registry(&cargo_config, &args)?;

    utils::update_index_packages(&cargo_config, &mut registry, &vec![list_args.name.clone()])?;

    let package_path = cargo_config
        .registry_index_path()
        .join(utils::registry_name(registry.source_id()))
        .join(".cache")
        .join(utils::package_dir(&list_args.name));

    let file_lock =
        package_path.open_ro(&list_args.name, &cargo_config, "Waiting for file lock...")?;

    let file_path = file_lock.path();
    if !file_path.exists() {
        anyhow::bail!("No package found with name {}", list_args.name);
    }

    let package_info = std::fs::read_to_string(file_path)?;
    let entries = create_package_entries(&package_info);

    if list_args.latest {
        print_latest(&entries);
    } else if list_args.yanked {
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
    let maybe_latest = entries
        .iter()
        .filter(|entry| !entry.yanked)
        .max_by(|e1, e2| e1.version.cmp(&e2.version));

    match maybe_latest {
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

// We get some useful lines along with some filler lines by splitting on '\u{0}'.
// We'll just discard the filler lines and return whatever successfully parses.
fn create_package_entries(package_info: &str) -> Vec<PackageEntry> {
    package_info
        .split('\u{0}')
        .map(serde_json::from_str)
        .flatten()
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageEntry {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: Version,
    #[serde(rename = "cksum")]
    pub checksum: String,
    pub yanked: bool,
}
