use crate::{
    cli::{ListArgs, Sorting},
    streaming_table::StreamingTable,
    IDable, ID_LEN,
};
use xdg_trash::{TrashFile, UnifiedTrash};

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

            if let Some(ref mut table) = table {
                table.draw_row([
                    id,
                    del_at,
                    &trash.to_string_lossy(),
                    &orig_path.to_string_lossy(),
                ]);
            } else {
                println!(
                    "{}\t{}\t{}\t{}\t",
                    id,
                    del_at,
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

            if let Some(ref mut table) = table {
                table.draw_row([id, del_at, &orig_path.to_string_lossy()]);
            } else {
                println!("{}\t{}\t{}\t", id, del_at, orig_path.display());
            }
        }
    }

    Ok(())
}
