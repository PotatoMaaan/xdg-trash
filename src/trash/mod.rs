use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

mod admin_trash;
mod home_trash;
mod operations;
mod user_trash;

#[derive(Debug)]
pub struct Trash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
    priority: i32,
    use_relative_path: bool,
}

impl Trash {
    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn info_dir(&self) -> &Path {
        &self.info_dir
    }

    pub fn files_dir(&self) -> &Path {
        &self.files_dir
    }

    pub fn mount_root(&self) -> &Path {
        &self.mount_root
    }

    pub fn device(&self) -> u64 {
        self.device
    }

    pub fn use_relative_path(&self) -> bool {
        self.use_relative_path
    }
}
