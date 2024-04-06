use std::{
    ffi::OsStr,
    fmt::Debug,
    fs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Component, Path, PathBuf},
    rc::Rc,
};
use trash_file::TrashFile;

#[cfg(test)]
mod test;

pub use error::*;
pub use trash::Trash;

pub mod error;

mod trash;
mod trash_file;
mod trashinfo;

#[derive(Debug)]
pub struct UnifiedTrash {
    trashes: Vec<Rc<Trash>>,
}

impl UnifiedTrash {
    /// Attempt to create a unified trash with all trashcans found in the system.
    pub fn new() -> crate::Result<Self> {
        let trashes = list_trashes()?.collect::<Vec<_>>();
        Ok(Self::new_with_trashcans(trashes))
    }

    /// Create a new unified trash with a custom selection of trashcans.
    /// An iterator over all trashcans can be obtained by the [`list_trashes`] function.
    pub fn new_with_trashcans(mut trashes: Vec<Rc<Trash>>) -> Self {
        sort_trashes(&mut trashes);
        Self { trashes }
    }

    /// Returns an iterator over all files in all trashcans
    pub fn list(&self) -> impl Iterator<Item = crate::Result<TrashFile>> + '_ {
        self.trashes
            .iter()
            .map(|trash| trash.clone().list())
            .flatten()
            .flatten()
    }

    /**
     * Attempts to trash the file into one of the known trashes, attempting to create
     * a new trashcan if one doesn't exists on the device.
     */
    pub fn put(&mut self, input_path: impl AsRef<Path>) -> crate::Result<TrashFile> {
        let input_path = input_path.as_ref();
        let input_path_meta = fs::symlink_metadata(input_path).map_err(|e| {
            crate::Error::FailedToTrashFile(
                input_path.to_owned(),
                Box::new(crate::Error::IoError(e)),
            )
        })?;

        let trash = if let Some(trash_on_same_dev) = self
            .trashes
            .iter()
            .find(|trash| trash.device() == input_path_meta.dev())
        {
            trash_on_same_dev.clone()
        } else {
            let mount_root = find_mount_root(&input_path).map_err(|e| {
                crate::Error::FailedToCreateTrash(input_path.to_owned(), Box::new(e))
            })?;
            let new_trash = Rc::new(Trash::create_user_trash(mount_root).map_err(|e| {
                crate::Error::FailedToCreateTrash(input_path.to_owned(), Box::new(e))
            })?);

            /*
             * We can push this into the trashes without re-sorting because user trashes
             * have the lowest priority, so it would end up somewhere at the end of the
             * list even if we sorted.
             */
            self.trashes.push(new_trash.clone());
            new_trash
        };

        trash.put(input_path)
    }

    /**
     * Attempts to delete all trashed files, returning an iterator over
     * the path of the deleted file or an error.
     */
    pub fn empty(&self) -> crate::Result<impl Iterator<Item = crate::Result<PathBuf>> + '_> {
        Ok(self
            .trashes
            .iter()
            .map(|trash| trash.empty())
            .flatten()
            .flatten())
    }
}

/// Returns an iterator over all trashes available on the system
pub fn list_trashes() -> crate::Result<impl Iterator<Item = Rc<Trash>>> {
    let home_trash =
        Trash::find_home_trash().map_err(|e| crate::Error::FailedToFindHomeTrash(Box::new(e)))?;

    let mounts_iter = list_mounts()?
        .into_iter()
        .map(|mount| -> Option<Rc<Trash>> {
            let admin_trash = Trash::find_admin_trash(mount.clone());
            let user_trash = Trash::find_user_trash(mount);

            admin_trash.ok().or(user_trash.ok()).map(Rc::new)
        })
        .flatten();

    Ok(mounts_iter.chain([Rc::new(home_trash)].into_iter()))
}

/// Sort trashes by their priority such that admin trashes will always be before user trashes
fn sort_trashes(trashes: &mut Vec<Rc<Trash>>) {
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

/// like fs::canonicalize but doesn't follow symlinks and doesn't check if the file exists.
///
/// Credit: <https://internals.rust-lang.org/t/path-to-lexical-absolute/14940>
fn lexical_absolute(p: &Path) -> std::io::Result<PathBuf> {
    let mut absolute = if p.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    for component in p.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                absolute.pop();
            }
            component => absolute.push(component.as_os_str()),
        }
    }
    Ok(absolute)
}

/// Finds the mount point of the filesystem on which the path resides
fn find_mount_root(path: &Path) -> crate::Result<PathBuf> {
    let path = path.canonicalize()?;
    let root_dev = fs::metadata(&path)?.dev();
    path.ancestors()
        .map(|p| (p, fs::metadata(p)))
        .map(|(p, x)| (p, x.map(|x| x.dev())))
        .take_while(|(_, x)| x.as_ref().ok() == Some(&root_dev))
        .map(|(p, x)| x.map(|_| p))
        .map(|x| x.map_err(|e| crate::Error::IoError(e)))
        .inspect(|x| println!("{:?}", x))
        .collect()
}
