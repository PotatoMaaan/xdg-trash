use super::Trash;
use crate::trash::TrashType;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::PathBuf,
};

impl Trash {
    pub fn find_admin_trash(mount_root: PathBuf) -> crate::Result<Self> {
        let trash_dir = mount_root.join(".Trash");
        let trash_dir_meta = fs::symlink_metadata(&trash_dir)?;
        let uid = unsafe { libc::getuid() };
        let uid = uid.to_string();

        if trash_dir_meta.permissions().mode() & 0o1000 != 0 {
            log::warn!(
                "Rejecting admin trash at {} because the sticky bit is not set",
                trash_dir.display()
            );
            return Err(crate::Error::NotSticky(trash_dir));
        }

        if trash_dir_meta.is_symlink() {
            log::warn!(
                "Rejecting admin trash at {} because it is a symlink",
                trash_dir.display()
            );
            return Err(crate::Error::IsSymlink(trash_dir));
        }

        let info_dir = trash_dir.join(&uid).join("info");
        let files_dir = trash_dir.join(&uid).join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        log::debug!("Found admin trash at: {}", trash_dir.display());

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root,
            info_dir,
            files_dir,
            trash_type: TrashType::Admin,
            use_relative_path: true,
        })
    }
}
