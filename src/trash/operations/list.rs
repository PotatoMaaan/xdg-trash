use crate::{trash::Trash, trash_file::TrashFile};
use std::{fs, rc::Rc};

impl Trash {
    /// Returns an iterator over all trashed files in this trashcan
    pub fn list(self: Rc<Self>) -> crate::Result<impl Iterator<Item = crate::Result<TrashFile>>> {
        let info_files = fs::read_dir(&self.info_dir)?;

        Ok(Box::new(info_files.map(
            move |info_file| -> crate::Result<TrashFile> {
                let info_file = info_file?;
                let info_file_path = info_file.path();

                TrashFile::from_trashinfo_path(&info_file_path, self.clone())
            },
        )))
    }
}
