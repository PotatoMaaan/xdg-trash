use crate::{trash::Trash, trashinfo::TrashInfo};
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Debug)]
pub struct TrashFile {
    trash: Rc<Trash>,
    trashinfo: TrashInfo,
    /// Filename in files WITHOUT .trashinfo ext
    raw_filename: OsString,
}

impl TrashFile {
    pub(crate) fn new_unchecked(
        trash: Rc<Trash>,
        trashinfo: TrashInfo,
        raw_filename: OsString,
    ) -> Self {
        Self {
            trash,
            trashinfo,
            raw_filename,
        }
    }

    pub fn from_trashinfo_path(info_file_path: &Path, trash: Rc<Trash>) -> crate::Result<Self> {
        let info_file = fs::read_to_string(info_file_path).map_err(|e| {
            crate::Error::InvalidTrashinfoFile(
                info_file_path.to_owned(),
                Box::new(crate::Error::IoError(e)),
            )
        })?;
        Self::from_trashinfo_file(&info_file, info_file_path, trash)
            .map_err(|e| crate::Error::InvalidTrashinfoFile(info_file_path.to_owned(), Box::new(e)))
    }

    /// This function has this a signature to make it testable without mocking a filesystem
    fn from_trashinfo_file(
        info_file: &str,
        info_file_path: &Path,
        trash: Rc<Trash>,
    ) -> crate::Result<Self> {
        if info_file_path.extension() != Some(OsStr::new("trashinfo")) {
            return Err(crate::Error::InvalidTrashinfoExt);
        }

        let trashinfo = info_file.parse::<TrashInfo>()?;

        let without_trashinfo_ext = info_file_path
            .file_stem()
            .ok_or(crate::Error::HasNoFileStem(info_file_path.to_owned()))?
            .to_owned();

        if !trash.files_dir().join(&without_trashinfo_ext).exists() {
            log::warn!("Orphaned trashinfo file: {}", info_file_path.display());
            return Err(crate::Error::OrphanedTrashinfoFile);
        }

        Ok(Self {
            trash,
            trashinfo,
            raw_filename: without_trashinfo_ext,
        })
    }

    /// The location this entry will be moved to when restored
    pub fn original_path(&self) -> PathBuf {
        if self.trashinfo.path.is_relative() {
            self.trash.mount_root().join(&self.trashinfo.path)
        } else {
            self.trashinfo.path.clone()
        }
    }

    /// Full path to this entrys entry in the files directory
    pub fn files_filepath(&self) -> PathBuf {
        self.trash.files_dir().join(&self.raw_filename)
    }

    /// Full path to this trash entrys .trashinfo file
    pub fn info_filepath(&self) -> PathBuf {
        let mut base_filename = self.raw_filename.clone();
        base_filename.push(".trashinfo");
        self.trash.info_dir().join(base_filename)
    }

    /// Permanently remove this file from the trash
    pub fn remove(self) -> crate::Result<()> {
        fs::remove_file(self.info_filepath())?;
        let file = self.files_filepath();
        let file_meta = fs::metadata(&file)?;
        if file_meta.is_dir() {
            fs::remove_dir_all(&file)?;
        } else {
            fs::remove_file(file)?;
        }
        Ok(())
    }

    /// Restore the file to it's original location
    pub fn restore(self) -> crate::Result<PathBuf> {
        let removed_path = self.original_path();
        fs::rename(self.files_filepath(), &removed_path)?;
        fs::remove_file(self.info_filepath())?;
        Ok(removed_path)
    }
}
