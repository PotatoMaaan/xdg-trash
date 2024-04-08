use super::Trash;
use crate::trash::TrashType;
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

impl Trash {
    /// Find a user-created trashcan (`.Trash-{uid}`) at the given mount root
    pub fn find_user_trash(mount_root: PathBuf) -> crate::Result<Self> {
        Self::user_trash_inner(mount_root, false)
    }

    /// Create a user-created trashcan (`.Trash-{uid}`) at the given mount root
    pub fn create_user_trash(mount_root: PathBuf) -> crate::Result<Self> {
        Self::user_trash_inner(mount_root, true)
    }

    fn user_trash_inner(mount_root: PathBuf, create: bool) -> crate::Result<Self> {
        let trash_dir = get_trash_dir(&mount_root);
        if create {
            fs::create_dir_all(&trash_dir)?;
        }
        let trash_dir_meta = fs::metadata(&trash_dir)?;

        let info_dir = trash_dir.join("info");
        let files_dir = trash_dir.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        if create {
            log::info!("Created user trash at: {}", trash_dir.display());
        } else {
            log::debug!("Found user trash at: {}", trash_dir.display());
        }

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root: mount_root.clone(),
            based_on: mount_root,
            info_dir,
            files_dir,
            trash_type: TrashType::User,
            use_relative_path: true,
        })
    }
}

fn get_trash_dir(mount_root: &Path) -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let mut trash_dir = ".Trash-".to_owned();
    trash_dir.push_str(&uid.to_string());
    mount_root.join(trash_dir)
}
