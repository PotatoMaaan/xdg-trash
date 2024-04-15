use crate::{trash::Trash, trashinfo::TrashInfo};
use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

/// A trashed file
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TrashFile {
    trash: Rc<Trash>,
    trashinfo: TrashInfo,
    /// Filename WITHOUT .trashinfo ext
    raw_filename: OsString,
    #[cfg(feature = "fs_extra")]
    size: Option<u64>,
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
            #[cfg(feature = "fs_extra")]
            size: None,
        }
    }

    /// Constructs a trash file from the given .trashinfo file path in the given trash.
    ///
    /// # Errors
    /// - no corresponding file exists in the trash
    /// - the file does not have a .trashinfo extension
    /// - the file does not have filestem
    /// - other io errors
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

        if fs::symlink_metadata(trash.files_dir().join(&without_trashinfo_ext)).is_err() {
            log::warn!("Orphaned trashinfo file: {}", info_file_path.display());
            return Err(crate::Error::OrphanedTrashinfoFile);
        }

        Ok(Self {
            trash,
            trashinfo,
            raw_filename: without_trashinfo_ext,
            #[cfg(feature = "fs_extra")]
            size: None,
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

    /// The time this item was moved into the trash
    ///
    /// The spec says that this *should* be local time, but it can't be guaranteed.
    pub fn deleted_at(&self) -> chrono::NaiveDateTime {
        self.trashinfo.deleted_at
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
    pub fn remove(self) -> Result<(), (Self, crate::Error)> {
        match remove_inner(&self) {
            Ok(_) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }

    /// Returns a reference to the trash this item is in
    pub fn trash(&self) -> &Trash {
        &self.trash
    }

    /// Restores the file to it's original location, creating all parent
    /// directories of the file if they don't exist anymore.
    ///
    /// Returns the location the file was restored to.
    pub fn restore(self, overwrite_existing: bool) -> Result<PathBuf, (Self, crate::Error)> {
        match restore_inner(&self, overwrite_existing) {
            Ok(v) => Ok(v),
            Err(e) => Err((self, e)),
        }
    }

    /// Gets the size on disk in bytes for this item.
    ///
    /// # Note
    /// This value is cached after the first call.
    #[cfg(feature = "fs_extra")]
    pub fn size(&self) -> Result<u64, fs_extra::error::Error> {
        if let Some(size) = self.size {
            Ok(size)
        } else {
            fs_extra::dir::get_size(self.files_filepath())
        }
    }

    /// Same as size, but *uncached*
    #[cfg(feature = "fs_extra")]
    pub fn size_uncached(&self) -> Result<u64, fs_extra::error::Error> {
        fs_extra::dir::get_size(self.files_filepath())
    }
}

fn remove_inner(file: &TrashFile) -> crate::Result<()> {
    let files_file = file.files_filepath();
    let file_meta = fs::symlink_metadata(&files_file)?;
    if file_meta.is_dir() {
        fs::remove_dir_all(&files_file)?;
    } else {
        fs::remove_file(files_file)?;
    }
    Ok(fs::remove_file(file.info_filepath())?)
}

fn restore_inner(file: &TrashFile, overwrite_existing: bool) -> crate::Result<PathBuf> {
    let original_path = file.original_path();

    if !overwrite_existing && fs::symlink_metadata(&original_path).is_ok() {
        return Err(crate::Error::AlreadyExists(original_path));
    }

    if let Some(parent) = original_path.parent() {
        assert!(parent.is_absolute());
        fs::create_dir_all(parent)?;
    }

    fs::rename(file.files_filepath(), &original_path)?;
    fs::remove_file(file.info_filepath())?;
    Ok(original_path)
}
