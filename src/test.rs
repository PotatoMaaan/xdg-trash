use std::{path::PathBuf, rc::Rc};

use crate::{list_trashes, trash, UnifiedTrash};

// #[test]
// fn test_list() {
//     microlog::init(log::LevelFilter::Trace);

//     let trashes = list_trashes()
//         .unwrap()
//         .inspect(|x| println!("{x:?}"))
//         .collect();

//     let trash = UnifiedTrash::new_with_trashcans(trashes);
//     let x = trash
//         .list()
//         .map(|x| x.unwrap())
//         .inspect(|x| println!("{:#?}", x.files_filepath()))
//         .inspect(|x| println!("{:#?}", x.info_filepath()))
//         .collect::<Vec<_>>();
// }

#[test]
fn test_put() {
    microlog::init(log::LevelFilter::Trace);

    let t = trash::Trash::find_user_trash(PathBuf::from("/home/potato/mount/storage")).unwrap();
    Rc::new(t).list().unwrap().for_each(|x| {
        dbg!(&x);
    });
}
