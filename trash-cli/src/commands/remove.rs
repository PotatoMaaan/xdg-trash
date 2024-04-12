use crate::{cli::RemoveArgs, user_input::choose, HashID};
use anyhow::Context;
use std::{
    io::{stdout, Write},
    path::Path,
};
use xdg_trash::UnifiedTrash;

pub fn remove(args: RemoveArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().context("Failed to init trash")?;
    println!("Listing files, this might take a moment.");

    let matches = trash
        .list()
        .inspect(|x| {
            log::debug!("Listing: {x:#?}");
            print!(".");
            stdout().flush().unwrap();
        })
        .filter_map(Result::ok)
        .filter(|x| x.id() == args.id_or_path || x.original_path() == Path::new(&args.id_or_path))
        .collect::<Vec<_>>();
    println!();
    println!();

    if matches.is_empty() {
        anyhow::bail!("No matching items found!");
    }

    let choice = choose(matches);
    let rmpath = choice.original_path();
    choice
        .remove()
        .map_err(|(_, e)| e)
        .context("Failed to remove file")?;

    println!("Removed {}", rmpath.display());

    Ok(())
}
