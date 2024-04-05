use super::Trash;
use std::{
    env, fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct HomeTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}

impl Trash {
    pub fn find_home_trash() -> crate::Result<Self> {
        let home_dir = PathBuf::from(env::var("HOME").map_err(|_| crate::Error::Homeless)?);

        let xdg_data_dir = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or(home_dir.join(".local").join("share"));

        let trash_dir = xdg_data_dir.join("Trash");
        let trash_dir_meta = fs::metadata(&xdg_data_dir)?;

        let info_dir = trash_dir.join("info");
        let files_dir = trash_dir.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        log::debug!("Found home trash at: {}", trash_dir.display());

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root: xdg_data_dir,
            info_dir,
            files_dir,
            priority: 3,
            use_relative_path: false,
        })
    }
}
