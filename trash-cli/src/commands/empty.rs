use crate::cli::EmptyArgs;
use anyhow::Context;
use chrono::{Days, Local};
use xdg_trash::UnifiedTrash;

pub fn empty(args: &EmptyArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().context("Failed to init trash")?;

    for file in trash.list().filter_map(Result::ok).filter(|file| {
        if let Some(before) = args.before {
            return file.deleted_at() < before;
        }

        if let Some(after) = args.after {
            return file.deleted_at() > after;
        }

        if let Some(keep) = args.keep {
            let Some(before) = Local::now().naive_local().checked_sub_days(Days::new(keep)) else {
                return false;
            };
            return file.deleted_at() < before;
        }

        true
    }) {
        let p = file.original_path();
        if args.dry_run {
            println!("Would remove: {}", p.display());
            continue;
        }

        let orig_path = file.original_path();
        if let Err((_, e)) = file.remove() {
            log::error!("Failed to remove file: {e}");
        } else {
            println!("Removed {}", orig_path.display());
        }
    }

    Ok(())
}
