use crate::trash_file::TrashFile;
use std::{fmt::Debug, fs, path::Path};

mod admin_trash;
mod home_trash;
mod uid_trash;

pub use admin_trash::AdminTrash;
pub use home_trash::HomeTrash;
pub use uid_trash::UidTrash;

pub trait Trash: Debug {
    /// The directory where trashed files are moved to
    fn files_dir(&self) -> &Path;
    /// The directory where .trashinfo files reside
    fn info_dir(&self) -> &Path;
    /// The device on which the trashcan resides
    fn device(&self) -> u64;
    /// The priority this trashcan should have over others (higher number is more)
    fn priority(&self) -> i8;
    /// The path on which relative paths in the .trashinfo file are based on
    fn mount_root(&self) -> &Path;

    /// Workaround for some dyn shenanigans (<https://stackoverflow.com/a/61654763>)
    fn as_dyn(&self) -> &dyn Trash;

    /// Returns an iterator over all trashed files in this trashcan
    fn list(&self) -> crate::Result<Box<dyn Iterator<Item = crate::Result<TrashFile>> + '_>> {
        let info_files = fs::read_dir(self.info_dir())?;

        Ok(Box::new(info_files.map(
            |info_file| -> crate::Result<TrashFile> {
                let info_file = info_file?;
                let info_file_path = info_file.path();

                TrashFile::from_trashinfo_path(&info_file_path, self.as_dyn())
            },
        )))
    }
}
