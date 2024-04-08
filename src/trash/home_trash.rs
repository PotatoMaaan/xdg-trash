use super::Trash;
use crate::trash::TrashType;
use std::{env, fs, os::unix::fs::MetadataExt, path::PathBuf};

impl Trash {
    /// Finds the users home trashcan (located at  `$XDG_DATA_HOME/.local/share/Trash`)
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
            based_on: home_dir,
            info_dir,
            files_dir,
            trash_type: TrashType::Home,
            use_relative_path: false,
        })
    }
}
