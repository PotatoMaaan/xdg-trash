use crate::{
    cli::RemoveArgs,
    commands::common::{choose, list_trashes_matching_status},
};
use anyhow::Context;

pub fn remove(args: RemoveArgs) -> anyhow::Result<()> {
    let matches = list_trashes_matching_status(&args.id_or_path)?;

    if matches.is_empty() {
        anyhow::bail!("No matching items found!");
    }

    let choice = choose(matches);
    let rmpath = choice.original_path();
    choice
        .remove()
        .map_err(|(_, e)| e)
        .context("Failed to remove file")?;
    println!();

    println!("Removed {}", rmpath.display());

    Ok(())
}
