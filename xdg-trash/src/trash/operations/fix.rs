use crate::{Trash, TrashFile};
use std::{fs, rc::Rc};

impl Trash {
    /// Removes broken trashinfo files.
    ///
    /// Returns the amount of removed files.
    pub fn fix(self: Rc<Self>) -> crate::Result<usize> {
        let info_files = fs::read_dir(&self.info_dir)?;

        let mut total = 0;
        for tfile in info_files {
            let tfile = tfile?;
            let tfile = tfile.path();
            if let Err(_) = TrashFile::from_trashinfo_path(&tfile, self.clone()) {
                log::info!("Removing: {}", tfile.display());
                fs::remove_file(&tfile)?;
                total += 1;
            }
        }

        Ok(total)
    }
}
