use super::Trash;
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct UidTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}

impl UidTrash {
    pub fn new(mount_root: PathBuf) -> crate::Result<Self> {
        let uid = unsafe { libc::getuid() };
        let mut trash_dir = ".Trash-".to_owned();
        trash_dir.push_str(&uid.to_string());
        let trash_dir = mount_root.join(trash_dir);
        let trash_dir_meta = fs::metadata(&trash_dir)?;

        let info_dir = trash_dir.join("info");
        let files_dir = trash_dir.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        log::debug!("Found uid trash at: {}", trash_dir.display());

        Ok(Self {
            device: trash_dir_meta.dev(),
            mount_root,
            info_dir,
            files_dir,
        })
    }
}

impl Trash for UidTrash {
    fn files_dir(&self) -> &Path {
        &self.files_dir
    }

    fn info_dir(&self) -> &Path {
        &self.info_dir
    }

    fn device(&self) -> u64 {
        self.device
    }

    fn priority(&self) -> i8 {
        1
    }

    fn mount_root(&self) -> &Path {
        &self.mount_root
    }

    fn as_dyn(&self) -> &dyn Trash {
        self
    }
}
