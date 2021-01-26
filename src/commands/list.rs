use cargo::{core::Summary, util::config::Config as CargoConfig};

use crate::{args::CargoSideloadListArgs, utils};

pub fn list(args: CargoSideloadListArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let mut registry = utils::create_registry(&cargo_config, &args.registry)?;

    utils::update_index(&cargo_config, &mut registry)?;
    let summaries = utils::package_summaries(&cargo_config, &mut registry, &args.name)?;

    if args.latest {
        print_latest(&summaries);
    } else {
        print_published(&summaries);
    }

    Ok(())
}

fn print_published(summaries: &[Summary]) {
    for summary in summaries {
        print_summary(summary);
    }
}

fn print_latest(summaries: &[Summary]) {
    let latest = utils::latest_version(summaries);

    match latest {
        Some(latest) => println!("{}", latest.version()),
        None => println!("Package not found"),
    }
}

fn print_summary(summary: &Summary) {
    println!(
        r#"{{
    "name": "{}",
    "version": "{}",
    "checksum": "{:?}",
}}"#,
        summary.name(),
        summary.version(),
        summary.checksum()
    )
}
