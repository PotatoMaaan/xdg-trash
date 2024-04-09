use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

mod admin_trash;
mod home_trash;
mod operations;
mod user_trash;

/// A single trashcan on the system.
///
/// ## Note about `mount_root`
/// It is strongly advised to only specify paths as mount_root that are actually at the
/// root of a mounted filesystem. This implementation does support trashes at non-fs-root
/// locations, however, this is only the case when these trashes are already part of the
/// known trashes. The [`crate::list_trashes`] function does **NOT** find these trashes!
/// It's most likely that other implementations will also not find any trashes at these locations.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Trash {
    device: u64,
    mount_root: PathBuf,
    based_on: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
    trash_type: TrashType,
    use_relative_path: bool,
}

/// The type of a trashcan
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// The type of this trashcan
    pub fn trash_type(&self) -> TrashType {
        self.trash_type
    }

    /// Directory where `.trashinfo` files are stored
    pub fn info_dir(&self) -> &Path {
        &self.info_dir
    }

    /// Directory where trashes files are stored
    pub fn files_dir(&self) -> &Path {
        &self.files_dir
    }

    /// Directory to which relative paths from trashed files will be joined to.
    ///
    /// # Example
    /// `/mnt/disk1`: Trashed files in `/mnt/disk1/.Trash-1000` might have a relative
    /// `original_path`, this will then get joined onto `/mnt/disk1`
    /// to produce an absolute path.
    pub fn mount_root(&self) -> &Path {
        &self.mount_root
    }

    /// Like mount_root, but only contains the *public* part of the path.
    /// This gets checked to test if a given file can be stored in this trashcan.
    ///
    /// # Example
    /// `/home/user/.local/share` -> `mount_root`, contains *trash specific* path
    ///
    /// `/home/user`              -> `based_on`, only contains *public* section
    ///
    /// Without this, `/home/user/Documents/some_file.txt` would not qualify for
    /// the home trash, even if they are the same device
    pub fn based_on(&self) -> &Path {
        &self.based_on
    }

    /// Defines if this trash should use relative paths in it's `.trashinfo` files.
    /// This is the case for trashcans on, for example, removeable devices in order
    /// to not have to depend on the device being mounted at the same location every time.
    pub fn use_relative_path(&self) -> bool {
        self.use_relative_path
    }

    /// The device id of the filesystem this trash resides on
    pub fn device(&self) -> u64 {
        self.device
    }
}
