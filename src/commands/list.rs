use cargo::{core::Summary, util::config::Config as CargoConfig};

use crate::{args::CargoSideloadListArgs, utils};

pub fn list(args: CargoSideloadListArgs) -> anyhow::Result<()> {
    let cargo_config = CargoConfig::default()?;
    let mut registry = utils::create_registry(&cargo_config, &args.registry)?;

    utils::update_index(&cargo_config, &mut registry)?;
    let summaries = utils::package_summaries(&cargo_config, &mut registry, &args.name)?;

    if args.latest {
        print_latest(&summaries, false);
    } else if args.latest_version {
        print_latest(&summaries, true);
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

fn print_latest(summaries: &[Summary], version_only: bool) {
    let latest_version = utils::latest_version(summaries);

    match (latest_version, version_only) {
        (Some(latest), true) => println!("{}", latest.version()),
        (Some(latest), false) => print_summary(latest),
        _ => println!("Package not found"),
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
