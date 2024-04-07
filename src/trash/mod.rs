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
    trash_type: TrashType,
    use_relative_path: bool,
}

/// The type of a trashcan
#[derive(Debug, Clone, Copy)]
pub enum TrashType {
    /// Located in the users home directory, ususally in `~/.local/share/Trash`
    Home,

    /// A "user created" Trashcan, Named like so: `.Trash-{uid}`
    User,

    /// A trashcan created by an admin at the root of a mounted filesystem, named `.Trash`.
    /// For a directory to quality, it must not be a symlink and must have the sticky bit set.
    Admin,
}

impl TrashType {
    /// The priority of one trashcan over another, if multiple ones qualify for a file
    pub fn priority(&self) -> i32 {
        match self {
            TrashType::Home => 3,
            TrashType::Admin => 2,
            TrashType::User => 1,
        }
    }
}

impl Trash {
    pub fn trash_type(&self) -> TrashType {
        self.trash_type
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
