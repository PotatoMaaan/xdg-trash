pub mod error;
pub use error::*;
use std::{ffi::OsStr, fmt::Debug, fs, os::unix::ffi::OsStrExt, path::PathBuf};
use trash::{AdminTrash, HomeTrash, UidTrash};
use trash_file::TrashFile;

#[cfg(test)]
mod test;

mod trash;
mod trash_file;
mod trashinfo;

pub use trash::Trash;

#[derive(Debug)]
pub struct UnifiedTrash {
    trashes: Vec<Box<dyn Trash>>,
}

impl UnifiedTrash {
    /// Attempt to create a unified trash with all trashcans found in the system.
    pub fn new() -> crate::Result<Self> {
        let mut trashes = list_trashes()?.collect::<Vec<_>>();
        sort_trashes(&mut trashes);

        Ok(Self { trashes })
    }

    /// Create a new unified trash with a custom selection of trashcans.
    /// An iterator over all trashcans can be obtained by the [`list_trashes`] function.
    pub fn new_with_trashcans(mut trashes: Vec<Box<dyn Trash>>) -> Self {
        sort_trashes(&mut trashes);
        Self { trashes }
    }

    /// Returns an iterator over all files in all trashcans
    pub fn list(&self) -> impl Iterator<Item = crate::Result<TrashFile>> {
        self.trashes
            .iter()
            .map(|trash| trash.list())
            .flatten()
            .flatten()
    }
}
/// Returns an iterator over all trashes available on the system
pub fn list_trashes() -> crate::Result<Box<dyn Iterator<Item = Box<dyn Trash>>>> {
    let home_trash =
        HomeTrash::new().map_err(|e| crate::Error::FailedToFindHomeTrash(Box::new(e)))?;

    let mounts_iter = list_mounts()?
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
        .flatten();

    let mounts: Box<dyn Iterator<Item = Box<dyn Trash>>> = Box::new(mounts_iter);
    let home_iter: Box<dyn Iterator<Item = Box<dyn Trash>>> = Box::new(
        ([home_trash])
            .into_iter()
            .map(|x| -> Box<dyn Trash> { Box::new(x) }),
    );

    Ok(Box::new(home_iter.chain(mounts)))
}

/// Sort trashes by their priority such that admin trashes will always be before uid trashes
fn sort_trashes(trashes: &mut Vec<Box<dyn Trash>>) {
    trashes.sort_by_key(|x| x.priority() * -1);
}

/// Lists all mounted filesystems (on linux)
#[cfg(target_os = "linux")]
fn list_mounts() -> crate::Result<Vec<PathBuf>> {
    fs::read("/proc/mounts")
        .map_err(|_| crate::Error::InvalidProcMounts)?
        .split(|x| *x == b'\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.split(|x| *x == b' ').nth(1))
        .map(|x| x.map(OsStr::from_bytes))
        .map(|x| x.map(PathBuf::from).ok_or(crate::Error::InvalidProcMounts))
        .collect()
}
