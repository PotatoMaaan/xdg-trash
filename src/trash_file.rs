use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use crate::{trash::Trash, trashinfo::TrashInfo};

#[derive(Debug)]
pub struct TrashFile<'t> {
    trash: &'t dyn Trash,
    trashinfo: TrashInfo,
    raw_filename: OsString,
}

impl<'t> TrashFile<'t> {
    pub fn from_trashinfo_path(info_file_path: &Path, trash: &'t dyn Trash) -> crate::Result<Self> {
        let info_file = fs::read_to_string(info_file_path).map_err(|e| {
            crate::Error::InvalidTrashinfoFile(
                info_file_path.to_owned(),
                Box::new(crate::Error::IoError(e)),
            )
        })?;
        Self::from_trashinfo_file(
            &info_file,
            info_file_path
                .file_stem()
                .ok_or(crate::Error::HasNoFileStem(info_file_path.to_owned()))?
                .to_owned(),
            trash,
        )
        .map_err(|e| crate::Error::InvalidTrashinfoFile(info_file_path.to_owned(), Box::new(e)))
    }

    fn from_trashinfo_file(
        info_file: &str,
        info_file_name: OsString,
        trash: &'t dyn Trash,
    ) -> crate::Result<Self> {
        let trashinfo = info_file.parse::<TrashInfo>()?;

        Ok(Self {
            trash,
            trashinfo,
            raw_filename: info_file_name,
        })
    }

    pub fn original_path(&self) -> PathBuf {
        if self.trashinfo.path.is_relative() {
            self.trash.mount_root().join(&self.trashinfo.path)
        } else {
            self.trashinfo.path.clone()
        }
    }

    pub fn files_path(&self) -> PathBuf {
        self.files_path().join(&self.raw_filename)
    }

    pub fn info_path(&self) -> PathBuf {
        let mut base_filename = self.raw_filename.clone();
        base_filename.push(".trashinfo");
        self.info_path().join(base_filename)
    }
}
