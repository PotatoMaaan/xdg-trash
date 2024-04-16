use crate::{
    cli::RestoreArgs,
    commands::common::{ask_yes_no, choose, list_trashes_matching_status},
};
use anyhow::Context;

pub fn restore(args: RestoreArgs) -> anyhow::Result<()> {
    let matches = list_trashes_matching_status(&args.id_or_path)?;

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

    println!("Restored {}", orig_path.display());

    Ok(())
}
