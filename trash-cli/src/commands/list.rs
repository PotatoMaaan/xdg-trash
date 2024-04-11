use crate::{
    cli::{ListArgs, Sorting},
    streaming_table::StreamingTable,
    IDable, ID_LEN,
};
use humansize::DECIMAL;
use xdg_trash::{TrashFile, UnifiedTrash};

pub fn list(args: ListArgs) -> anyhow::Result<()> {
    let trash = UnifiedTrash::new().unwrap();

    let mut total_size = 0;

    let list = || -> Box<dyn Iterator<Item = TrashFile>> {
        let list = trash
            .list()
            .inspect(|x| {
                if let Err(e) = x {
                    log::error!("{}", e);
                }
            })
            .filter_map(Result::ok)
            .inspect(|x| {
                total_size += x.size().unwrap_or(0);
            });

        if let Some(sorting) = args.sort {
            let mut vec = list.collect::<Vec<_>>();
            vec.sort_by(|a, b| match sorting {
                Sorting::Trash => a.trash().mount_root().cmp(b.trash().mount_root()),
                Sorting::Path => a.original_path().cmp(&b.original_path()),
                Sorting::Date => a.deleted_at().cmp(&b.deleted_at()),
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

    if args.trash_location {
        let mut list = list();
        let ft = list
            .next()
            .map(|x| x.trash().mount_root().as_os_str().len() + 5);

        let mut table = if !args.simple {
            println!();
            Some(StreamingTable::draw_header([
                ("ID", Some(ID_LEN)),
                ("Deleted at", Some(19)),
                ("Size", Some(10)),
                ("Trash location", ft),
                ("Original Location", None),
            ]))
        } else {
            None
        };

        for file in list {
            let id = &file.id();
            let del_at = &file.deleted_at().to_string();
            let trash = &file.trash().mount_root();
            let orig_path = &file.original_path();
            let size = file.size();

            if let Some(ref mut table) = table {
                let size = size.map(|x| humansize::format_size(x, DECIMAL));
                table.draw_row([
                    id,
                    del_at,
                    &size.unwrap_or_else(|_| "N/A".to_owned()),
                    &trash.to_string_lossy(),
                    &orig_path.to_string_lossy(),
                ]);
            } else {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t",
                    id,
                    del_at,
                    &size
                        .map(|x| x.to_string())
                        .unwrap_or_else(|_| "N/A".to_owned()),
                    trash.display(),
                    orig_path.display()
                );
            }
        }
    } else {
        let mut table = if !args.simple {
            println!();
            Some(StreamingTable::draw_header([
                ("ID", Some(ID_LEN)),
                ("Deleted at", Some(19)),
                ("Size", Some(10)),
                ("Original Location", None),
            ]))
        } else {
            None
        };
        let list = list();

        for file in list {
            let id = &file.id();
            let del_at = &file.deleted_at().to_string();
            let orig_path = &file.original_path();
            let size = file.size();

            if let Some(ref mut table) = table {
                let size = size.map(|x| humansize::format_size(x, DECIMAL));
                table.draw_row([
                    id,
                    del_at,
                    &size.unwrap_or("N/A".to_owned()),
                    &orig_path.to_string_lossy(),
                ]);
            } else {
                println!(
                    "{}\t{}\t{}\t,{}\t",
                    id,
                    del_at,
                    size.map(|x| x.to_string()).unwrap_or("N/A".to_owned()),
                    orig_path.display()
                );
            }
        }
    }

    if !args.simple {
        println!();
        println!(
            "Total size: {}",
            humansize::format_size(total_size, DECIMAL)
        );
    }

    Ok(())
}
