use crate::{
    cli::{ListArgs, Sorting},
    streaming_table::StreamingTable,
    HashID, ID_LEN,
};
use humansize::DECIMAL;
use xdg_trash::{TrashFile, UnifiedTrash};

#[derive(Debug)]
enum TableDisplay<A, B> {
    NoTrash(A),
    WithTrash(B),
}

pub fn list(args: ListArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().unwrap();

    let list = || -> Box<dyn Iterator<Item = TrashFile>> {
        let list = trash
            .list()
            .inspect(|x| {
                if let Err(e) = x {
                    log::error!("{}", e);
                }
            })
            .filter_map(Result::ok);

        if let Some(sorting) = args.sort {
            let mut vec = list.collect::<Vec<_>>();
            vec.sort_by(|a, b| match sorting {
                Sorting::Trash => a.trash().mount_root().cmp(b.trash().mount_root()),
                Sorting::Path => a.original_path().cmp(&b.original_path()),
                Sorting::Date => a.deleted_at().cmp(&b.deleted_at()),
                // TODO? Replacing the size with zero upon failure might not be the best option here
                Sorting::Size => a.size().unwrap_or(0).cmp(&b.size().unwrap_or(0)),
            });

            if args.reverse {
                vec.reverse();
            }

            Box::new(vec.into_iter())
        } else {
            Box::new(list)
        }
    };

    let (table, list) = match (args.trash_location, args.simple) {
        (true, false) => {
            let mut list = list();
            let ft = list
                .next()
                .map(|x| x.trash().mount_root().as_os_str().len() + 5);
            (
                Some(TableDisplay::WithTrash(StreamingTable::draw_header([
                    ("ID", Some(ID_LEN)),
                    ("Deleted at", Some(19)),
                    ("Size", Some(8)),
                    ("Trash location", ft),
                    ("Original Location", None),
                ]))),
                list,
            )
        }
        (false, false) => {
            let list = list();
            (
                Some(TableDisplay::NoTrash(StreamingTable::draw_header([
                    ("ID", Some(ID_LEN)),
                    ("Deleted at", Some(19)),
                    ("Size", Some(8)),
                    ("Original Location", None),
                ]))),
                list,
            )
        }
        (_, true) => {
            let list = list();
            (None, list)
        }
    };

    let mut total_size = if args.size { Some(0) } else { None };

    for file in list {
        let id = &file.id();
        let del_at = &file.deleted_at().to_string();
        let orig_path = &file.original_path();
        let size = total_size.as_mut().and_then(|total_size| {
            let s = file.size().ok();
            if let Some(s) = s {
                *total_size += s;
            }
            s
        });
        let size_human = size
            .map(|x| humansize::format_size(x, DECIMAL))
            .unwrap_or_else(|| "N/A".to_owned());
        let trash = &file.trash().mount_root();

        match table {
            Some(TableDisplay::NoTrash(ref table)) => {
                table.draw_row([
                    id,
                    del_at,
                    &size_human,
                    orig_path.to_string_lossy().as_ref(),
                ]);
            }
            Some(TableDisplay::WithTrash(ref table)) => {
                table.draw_row([
                    id,
                    del_at,
                    &size_human,
                    &trash.to_string_lossy(),
                    &orig_path.to_string_lossy(),
                ]);
            }
            None => {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    id,
                    del_at,
                    size.map(|x| x.to_string())
                        .unwrap_or_else(|| "N/A".to_owned()),
                    trash.display(),
                    orig_path.display()
                );
            }
        }
    }

    if let Some(total_size) = total_size {
        if !args.simple {
            println!();
            println!(
                "Total size: {}",
                humansize::format_size(total_size, DECIMAL)
            );
        }
    }

    Ok(())
}
