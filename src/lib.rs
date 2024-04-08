//! Interact with xdg-trash implementations, see <https://specifications.freedesktop.org/trash-spec/trashspec-1.0.html>
//! 
//! ## Linux only
//! This crate is linux only for now, as it relies on reading `/proc/mounts` and uses some unix-only io extensions.
//! 
//! Trashcans can be located across multiple locations and physical devices, this is to avoid having to copy files
//! across filesystem boundaries upon trashing a file. This crate proides a [`UnifiedTrash`], which combines all
//! trashcans across the system into a single interface.
//! 
//! ## Considerations
//! When dealing with a users trashed files, it's probably a good idea to not always abort
//! on the first error, but to instread be fault tolerant in order to still provide functionality,
//! even if errors were encountered.
//! 
//! In practice this mostly means filetering out errors and or informing a user about a failure and
//! allowing them to choose further actions
//! 
//! # Example
//! This example shows how to trash a file and list all trashed files
//! ```
//! use xdg_trash::UnifiedTrash;
//! use std::fs::File;
//! 
//! let mut trash = UnifiedTrash::new().unwrap();
//! 
//! _ = File::create("somefile.txt");
//! trash.put("somefile.txt").unwrap();
//! 
//! for file in trash.list() {
//!     let file = file.unwrap();
//!     println!("Found in trash: {}", file.original_path().display());
//! }
//! ```

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
pub use trash::{Trash, TrashType};

pub mod error;

mod trash;
mod trash_file;
mod trashinfo;

/// Unifies all trashcans on the system into one interface.
#[derive(Debug)]
pub struct UnifiedTrash {
    known_trashes: Vec<Rc<Trash>>,
}

impl UnifiedTrash {
    /// Attempt to create a unified trash with all trashcans found in the system.
    pub fn new() -> crate::Result<Self> {
        let trashes = list_trashes()?;
        Ok(Self::with_trashcans(trashes))
    }

    /// Create a new unified trash with a custom selection of trashcans.
    /// An iterator over all trashcans can be obtained by the [`list_trashes`] function.
    ///
    /// Note that some function (such as [`Self::put`]) might still use / create new trashcans.
    /// Use the `_known` functions ([`Self::put_known`]) to only use this list of trashcans
    ///
    /// # Examples
    /// ```
    /// use xdg_trash::{list_trashes, UnifiedTrash};
    ///
    /// // Filter out trashes on a specific device
    /// let trashes = list_trashes().unwrap().filter(|t| t.device() != 500);
    /// let unified_trash = UnifiedTrash::with_trashcans(trashes);
    /// ```
    pub fn with_trashcans(trashes: impl Iterator<Item = Rc<Trash>>) -> Self {
        let mut trashes = trashes.collect::<Vec<_>>();
        sort_trashes(&mut trashes);
        Self {
            known_trashes: trashes,
        }
    }

    /// Returns an iterator over all files in all *known* trashcans.
    /// 
    /// The iterator will yield an error if a `.trashinfo` file has no correspondig actual file,
    /// so you might want to simply filter out all errors.
    pub fn list(&self) -> impl Iterator<Item = crate::Result<TrashFile>> + '_ {
        self.known_trashes
            .iter()
            .map(|trash| trash.clone().list())
            .flatten()
            .flatten()
    }

    /// Puts the file at `input_path` into one of the *known* trashcans, failing if no matching trashcan is known.
    pub fn put_known(&mut self, input_path: impl AsRef<Path>) -> crate::Result<TrashFile> {
        self.put_inner(input_path.as_ref(), true)
    }

    /// Attempts to put the file at `input_path` into a trashcan, creating a new one if one doesn't exist.
    pub fn put(&mut self, input_path: impl AsRef<Path>) -> crate::Result<TrashFile> {
        self.put_inner(input_path.as_ref(), false)
    }

    fn put_inner(&mut self, input_path: &Path, known_only: bool) -> crate::Result<TrashFile> {
        let input_path_meta = fs::symlink_metadata(input_path).map_err(|e| {
            crate::Error::FailedToTrashFile(
                input_path.to_owned(),
                Box::new(crate::Error::IoError(e)),
            )
        })?;
        let input_abs = lexical_absolute(input_path)?;
        let trash = if let Some(known_trash) = self.known_trashes.iter().inspect(|x| log::trace!("Checking: {:?}", x.mount_root())).find(|trash| {
            // Checks if file is on the same physical device as the trashcan
            trash.device() == input_path_meta.dev()
                // checks if the file is a child of the trash mount root, if this is not the case,
                // it means that multiple trashes exist on the same device. In this case, we just continue searching
                && input_abs.strip_prefix(trash.based_on()).is_ok()
        }) {
            log::trace!("Found matching trash");
            known_trash.clone()
        } else if known_only {
            return Err(crate::Error::NoTrashFound);
        } else {
            log::trace!("No trash found, trying to find or create one");

            let mount_root = find_mount_root(&input_path)?;

            let trash = if let Some(found_trash) = find_any_trash_at(mount_root.clone()) {
                found_trash
            } else {
                Trash::create_user_trash(mount_root).map_err(|e| {
                    crate::Error::FailedToCreateTrash(input_path.to_owned(), Box::new(e))
                })?
            };

            let trash = Rc::new(trash);

            /*
             * We can push this into the trashes without re-sorting because user trashes
             * have the lowest priority, so it would end up somewhere at the end of the
             * list even if we sorted.
             */
            self.known_trashes.push(trash.clone());
            trash
        };

        log::trace!("Putting into trash");
        trash.put(input_path)
    }

    /// Attempts to delete all trashed files in the *known* trash cans.
    pub fn empty(&self) -> crate::Result<impl Iterator<Item = crate::Result<PathBuf>> + '_> {
        Ok(self
            .known_trashes
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
    let mounts_iter = list_mounts()?.into_iter().map(find_any_trash_at).flatten();

    Ok(mounts_iter.chain([home_trash].into_iter()).map(Rc::new))
}

fn find_any_trash_at(mount_root: PathBuf) -> Option<Trash> {
    let admin_trash = Trash::find_admin_trash(mount_root.clone());
    let user_trash = Trash::find_user_trash(mount_root);

    admin_trash.ok().or(user_trash.ok())
}

/// Sort trashes by their priority such that admin trashes will always be before user trashes
fn sort_trashes(trashes: &mut Vec<Rc<Trash>>) {
    trashes.sort_by_key(|x| x.trash_type().priority() * -1);
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
    let path = lexical_absolute(path)?;
    let root_dev = fs::symlink_metadata(&path)?.dev();
    path.ancestors()
        .map(|p| (p, fs::metadata(p)))
        .map(|(p, x)| (p, x.map(|x| x.dev())))
        .take_while(|(_, x)| x.as_ref().ok() == Some(&root_dev))
        .map(|(p, x)| x.map(|_| p))
        .map(|x| x.map_err(|e| crate::Error::IoError(e)))
        .collect()
}
