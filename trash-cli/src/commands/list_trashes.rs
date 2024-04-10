use anyhow::Context;

use crate::{cli::ListTrashesArgs, streaming_table::StreamingTable};

pub fn list_trashes(args: ListTrashesArgs) -> anyhow::Result<()> {
    let trashes = xdg_trash::list_trashes().context("Failed to list trashes")?;

    let table =
        StreamingTable::draw_header([("Device ID", Some(9)), ("Type", Some(5)), ("Path", None)]);

    for trash in trashes {
        table.draw_row([
            &trash.device().to_string(),
            &trash.trash_type().to_string(),
            trash.mount_root().to_string_lossy().as_ref(),
        ]);
    }

    Ok(())
}
