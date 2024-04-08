use crate::{Trash, UnifiedTrash};
use dircpy::copy_dir;
use std::{fs, path::PathBuf, rc::Rc};
use tempdir::TempDir;

fn prepare_testdir() -> (TempDir, [PathBuf; 2], [Rc<Trash>; 2]) {
    let tmpdir = TempDir::new("xdg_trash_test").unwrap();
    let p = tmpdir.path();
    let dir1 = p.join("dir1");
    let dir2 = p.join("dir2");

    for dir in [&dir1, &dir2] {
        copy_dir("test_files", dir).unwrap();
    }

    let trash1 = Trash::create_user_trash(dir1.clone()).unwrap();
    let trash2 = Trash::create_user_trash(dir2.clone()).unwrap();

    (tmpdir, [dir1, dir2], [Rc::new(trash1), Rc::new(trash2)])
}

#[test]
fn test_single_trash_put_list_restore() {
    let (_tmpdir, dirs, trashes) = prepare_testdir();
    let f1 = dirs[0].join("symlink");
    let t1 = trashes[0].clone();

    assert!(f1.exists());
    t1.clone().put(&f1).unwrap();
    assert!(!f1.exists());

    t1.list()
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .restore(true)
        .unwrap();
    assert!(f1.exists());
}

#[test]
fn test_single_trash_put_multiple() {
    let (_tmpdir, dirs, trashes) = prepare_testdir();
    let f1 = dirs[0].join("Text File.txt");
    let t1 = trashes[0].clone();
    let mut f2 = f1.clone();
    f2.set_file_name("file_copy");

    dbg!(&f1);
    dbg!(&f2);
    fs::copy(&f1, &f2).unwrap();
    assert!(f1.exists());
    assert!(f2.exists());

    t1.clone().put(&f1).unwrap();
    assert!(!f1.exists());

    fs::rename(&f2, &f1).unwrap();
    assert!(f1.exists());

    t1.clone().put(&f1).unwrap();

    assert!(!f1.exists());
}

#[test]
fn test_single_trash_put_list_remove() {
    let (_tmpdir, dirs, trashes) = prepare_testdir();
    let f1 = dirs[0].join("symlink");
    let t1 = trashes[0].clone();

    assert!(f1.exists());
    t1.clone().put(&f1).unwrap();
    assert!(!f1.exists());

    t1.list()
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .remove()
        .unwrap();
    assert!(!f1.exists());
}

#[test]
fn test_put_list_restore() {
    _ = microlog::try_init(log::LevelFilter::Trace);

    let (_tmpdir, dirs, trashes) = prepare_testdir();
    let mut unified = UnifiedTrash::with_trashcans(trashes.into_iter());

    let f1 = dirs[0].join("symlink");
    let f2 = dirs[0].join("Text File.txt");
    let f3 = dirs[1].join("trash1.pdf");
    let f4 = dirs[1].join("some dir");

    assert!(f1.exists());
    assert!(f2.exists());
    assert!(f3.exists());
    assert!(f4.exists());

    unified.put_known(&f1).unwrap();
    unified.put_known(&f2).unwrap();
    unified.put_known(&f3).unwrap();
    unified.put_known(&f4).unwrap();

    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(!f3.exists());
    assert!(!f4.exists());

    let listed = unified.list().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(listed.len(), 4);

    listed.iter().find(|x| x.original_path() == f1).unwrap();
    listed.iter().find(|x| x.original_path() == f2).unwrap();
    listed.iter().find(|x| x.original_path() == f3).unwrap();
    listed.iter().find(|x| x.original_path() == f4).unwrap();

    listed.into_iter().for_each(|x| {
        x.restore(true).unwrap();
    });

    assert!(f1.exists());
    assert!(f2.exists());
    assert!(f3.exists());
    assert!(f4.exists());

    assert!(f1.is_symlink());
}

#[test]
fn test_put_list_remove() {
    _ = microlog::try_init(log::LevelFilter::Trace);

    let (_tmpdir, dirs, trashes) = prepare_testdir();
    let mut unified = UnifiedTrash::with_trashcans(trashes.iter().cloned());

    let f1 = dirs[0].join("symlink");
    let f2 = dirs[0].join("Text File.txt");
    let f3 = dirs[1].join("trash1.pdf");
    let f4 = dirs[1].join("some dir");

    assert!(f1.exists());
    assert!(f2.exists());
    assert!(f3.exists());
    assert!(f4.exists());

    unified.put_known(&f1).unwrap();
    unified.put_known(&f2).unwrap();
    unified.put_known(&f3).unwrap();
    unified.put_known(&f4).unwrap();

    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(!f3.exists());
    assert!(!f4.exists());

    let listed = unified.list().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(listed.len(), 4);

    listed.iter().find(|x| x.original_path() == f1).unwrap();
    listed.iter().find(|x| x.original_path() == f2).unwrap();
    listed.iter().find(|x| x.original_path() == f3).unwrap();
    listed.iter().find(|x| x.original_path() == f4).unwrap();

    listed.into_iter().for_each(|x| {
        x.remove().unwrap();
    });

    assert!(!f1.exists());
    assert!(!f2.exists());
    assert!(!f3.exists());
    assert!(!f4.exists());

    trashes.iter().for_each(|trash| {
        assert_eq!(fs::read_dir(trash.files_dir()).unwrap().count(), 0);
        assert_eq!(fs::read_dir(trash.info_dir()).unwrap().count(), 0);
    });
}
