use crate::trash::Trash;
use std::{fs, path::PathBuf};

impl Trash {
    pub fn empty(&self) -> crate::Result<impl Iterator<Item = crate::Result<PathBuf>>> {
        empty_inner(self)
    }
}

fn empty_inner(trash: &Trash) -> crate::Result<impl Iterator<Item = crate::Result<PathBuf>>> {
    let infos = fs::read_dir(&trash.info_dir)?;
    let files = fs::read_dir(&trash.files_dir)?;
    Ok(infos
        .chain(files)
        .flat_map(|x| {
            x.map(|entry| {
                let path = entry.path();
                entry.file_type().map(|x| {
                    if x.is_dir() {
                        fs::remove_dir_all(path)
                    } else {
                        fs::remove_file(path)
                    }
                    .map(|_| entry.path())
                    .map_err(crate::Error::IoError)
                })
            })
        })
        .flatten())
}
