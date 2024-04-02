pub mod error;
pub use error::*;
use std::{ffi::OsStr, fmt::Debug, fs, os::unix::ffi::OsStrExt, path::PathBuf};
use trash::{AdminTrash, HomeTrash, Trash, UidTrash};
use trash_file::TrashFile;

#[cfg(test)]
mod test;

mod trash;
mod trash_file;
mod trashinfo;

#[derive(Debug)]
pub struct UnifiedTrash {
    trashes: Vec<Box<dyn Trash>>,
}

impl UnifiedTrash {
    pub fn new() -> crate::Result<Self> {
        let home_trash =
            HomeTrash::new().map_err(|e| crate::Error::FailedToFindHomeTrash(Box::new(e)))?;

        let mut trashes = list_mounts()?
            .into_iter()
            .map(|mount| -> Option<Box<dyn Trash>> {
                let admin_trash = AdminTrash::new(mount.clone());
                let uid_trash = UidTrash::new(mount);

                if let Ok(t) = admin_trash {
                    return Some(Box::new(t));
                }
                if let Ok(t) = uid_trash {
                    return Some(Box::new(t));
                }

                None
            })
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        trashes.insert(0, Box::new(home_trash));

        // Sort trashes by their priority such that admin trashes will always be before uid trashes
        trashes.sort_by_key(|x| x.priority() * -1);

        Ok(Self { trashes })
    }

    pub fn list(&self) -> impl Iterator<Item = crate::Result<TrashFile>> {
        self.trashes
            .iter()
            .map(|trash| trash.list())
            .flatten()
            .flatten()
    }
}

fn list_mounts() -> crate::Result<Vec<PathBuf>> {
    fs::read("/proc/mounts")?
        .split(|x| *x as char == '\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.split(|x| *x == b' ').nth(1))
        .map(|x| x.map(OsStr::from_bytes))
        .map(|x| x.map(PathBuf::from).ok_or(crate::Error::InvalidProcMounts))
        .collect()
}
