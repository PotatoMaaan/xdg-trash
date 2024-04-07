use crate::{list_trashes, trash, Trash, UnifiedTrash};
use std::{fs, io, path::Path};
use tempdir::TempDir;

// #[test]
// fn test_list() {
//     microlog::init(log::LevelFilter::Trace);

//     let trash = UnifiedTrash::new().unwrap();
//     let item = trash.list().nth(10).unwrap().unwrap();
//     dbg!(&item.original_path());
//     item.remove().unwrap();
// }

pub fn copy_recursive(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursive(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn real() -> TempDir {
    let tmpdir = TempDir::new("xdg_trash_test").unwrap();
    let p = tmpdir.path();
    let dir1 = p.join("dir1");
    let dir2 = p.join("dir2");

    dbg!(&dir1);

    for dir in [&dir1, &dir2] {
        copy_recursive("test_files", dir).unwrap();
    }

    dbg!(&fs::read_dir(&dir1));

    let trash1 = Trash::create_user_trash(dir1).unwrap();
    let trash2 = Trash::create_user_trash(dir2).unwrap();

    let unified = UnifiedTrash::with_trashcans([trash1, trash2].into_iter());

    tmpdir
}

#[test]
fn test() {
    let x = real();
}

// #[test]
// fn test_put() {
//     microlog::init(log::LevelFilter::Trace);

//     let t = trash::Trash::find_user_trash(PathBuf::from("/home/potato/mount/storage")).unwrap();
//     Rc::new(t).list().unwrap().for_each(|x| {
//         dbg!(&x);
//     });
// }
