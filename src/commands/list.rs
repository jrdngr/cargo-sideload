use crate::args::CargoSideloadArgs;

pub fn list(_args: CargoSideloadArgs) -> anyhow::Result<()> {
    println!("Listing crates");

    Ok(())
}
