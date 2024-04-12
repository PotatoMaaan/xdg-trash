use crate::{
    cli::RestoreArgs,
    user_input::{ask_yes_no, choose},
    HashID,
};
use anyhow::Context;
use std::{
    io::{stdout, Write},
    path::Path,
};
use xdg_trash::UnifiedTrash;

pub fn restore(args: RestoreArgs) -> anyhow::Result<()> {
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
    let orig_path = choice.original_path();

    if let Err((choice, e)) = choice.restore(false) {
        match e {
            xdg_trash::Error::AlreadyExists(ref p) => {
                println!("A file already exists at {}\n", p.display());
                if ask_yes_no("Do you wan to overwrite it?", false) {
                    choice
                        .restore(true)
                        .map_err(|(_, e)| e)
                        .with_context(|| format!("Failed to restore file: {}", e))?;
                    println!("Restored  {}", orig_path.display());
                } else {
                    log::error!("Cancelled by user");
                }
            }
            _ => anyhow::bail!("Failed to restore file: {e}"),
        }
    }

    Ok(())
}
