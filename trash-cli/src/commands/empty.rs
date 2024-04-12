use crate::cli::EmptyArgs;
use anyhow::Context;
use xdg_trash::UnifiedTrash;

pub fn empty(args: EmptyArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().context("Failed to init trash")?;
    trash
        .empty()
        .context("Failed to emptry trash")?
        .for_each(|deleted| match deleted {
            Ok(deleted) => {
                println!("Removed {}", deleted.display());
            }
            Err(e) => {
                log::error!("{e}");
            }
        });

    Ok(())
}
