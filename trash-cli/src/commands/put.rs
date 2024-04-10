use crate::cli::PutArgs;
use anyhow::Context;
use std::rc::Rc;
use xdg_trash::{Trash, UnifiedTrash};

pub fn put(args: PutArgs) -> anyhow::Result<()> {
    let home_trash =
        Trash::find_or_create_home_trash().context("Failed to init the home trashcan")?;

    let mut trash = UnifiedTrash::with_trashcans([Rc::new(home_trash)].into_iter());

    for file in args.files {
        match trash.put(&file) {
            Ok(_) => {
                println!("Trashed {}", file.display());
            }
            Err(e) => {
                if args.force {
                    log::error!("{}", e);
                } else {
                    return Err(anyhow::anyhow!(e));
                }
            }
        }
    }

    Ok(())
}
