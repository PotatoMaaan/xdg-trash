use crate::{cli::ListTrashesArgs, streaming_table::StreamingTable};
use anyhow::Context;

pub fn list_trashes(args: &ListTrashesArgs) -> anyhow::Result<()> {
    let trashes = xdg_trash::list_trashes().context("Failed to list trashes")?;

    let table = if !args.simple {
        Some(StreamingTable::draw_header([
            ("Device ID", Some(9)),
            ("Type", Some(5)),
            ("Path", Some(15)),
        ]))
    } else {
        None
    };

    for trash in trashes {
        let dev = trash.device().to_string();
        let trash_type = trash.trash_type().to_string();
        let location = trash
            .info_dir()
            .parent()
            .context("Info dir has no parent dir")?
            .to_string_lossy();

        if let Some(ref table) = table {
            table.draw_row([&dev, &trash_type, &location]);
        } else {
            println!("{}\t{}\t{}\t", dev, trash_type, location);
        }
    }

    Ok(())
}
