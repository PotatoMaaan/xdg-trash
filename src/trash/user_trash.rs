use super::Trash;
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct UserTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}

impl Trash {
    pub fn find_user_trash(mount_root: PathBuf) -> crate::Result<Self> {
        let trash_dir = get_trash_dir(&mount_root);
        let trash_dir_meta = fs::metadata(&trash_dir)?;

        let info_dir = trash_dir.join("info");
        let files_dir = trash_dir.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        log::debug!("Found user trash at: {}", trash_dir.display());

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root,
            info_dir,
            files_dir,
            priority: 1,
            use_relative_path: true,
        })
    }

    pub fn create_user_trash(mount_root: PathBuf) -> crate::Result<Self> {
        let trash_dir = get_trash_dir(&mount_root);
        fs::create_dir_all(&trash_dir)?;
        let trash_dir_meta = fs::metadata(&trash_dir)?;

        let info_dir = trash_dir.join("info");
        let files_dir = trash_dir.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        log::info!("Created user trash at: {}", trash_dir.display());

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root,
            info_dir,
            files_dir,
            priority: 1,
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
