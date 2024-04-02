pub mod error;
pub use error::*;
use std::{
    env,
    ffi::OsStr,
    fmt::Debug,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    time::SystemTime,
};

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct UnifiedTrash {
    trashes: Vec<Box<dyn Trash>>,
}

#[derive(Debug)]
struct HomeTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}

#[derive(Debug)]
struct AdminTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}

#[derive(Debug)]
struct UidTrash {
    device: u64,
    mount_root: PathBuf,
    info_dir: PathBuf,
    files_dir: PathBuf,
}
trait Trash: Debug {
    fn files_dir(&self) -> &Path;
    fn info_dir(&self) -> &Path;
    fn device(&self) -> u64;
    fn priority(&self) -> i8;

    fn list(&self) -> std::io::Result<fs::ReadDir> {
        fs::read_dir(self.files_dir())
    }
}

#[derive(Debug)]
pub struct TrashFile<'t> {
    trash: &'t dyn Trash,
    path: PathBuf,
    modified: SystemTime,
}

impl HomeTrash {
    pub fn new() -> crate::Result<Self> {
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
        })
    }
}

impl AdminTrash {
    pub fn new(mount_root: PathBuf) -> crate::Result<Self> {
        let trash_dir = mount_root.join(".Trash");
        let trash_dir_meta = fs::metadata(&trash_dir)?;
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
        })
    }
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

impl Trash for HomeTrash {
    fn device(&self) -> u64 {
        self.device
    }

    fn files_dir(&self) -> &Path {
        &self.files_dir
    }

    fn info_dir(&self) -> &Path {
        &self.info_dir
    }

    fn priority(&self) -> i8 {
        3
    }
}

impl Trash for AdminTrash {
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
        2
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
}

impl UnifiedTrash {
    pub fn new() -> crate::Result<Self> {
        let home_trash =
            HomeTrash::new().map_err(|e| crate::Error::FailedToFindHomeTrash(Box::new(e)))?;

        let mut trashes = list_mounts()?
            .into_iter()
            .map(|mount| -> Option<Box<dyn Trash>> {
                let admin_trash = AdminTrash::new(mount.clone());
                let uid_trash = UidTrash::new(mount);

                if let Ok(t) = admin_trash {
                    return Some(Box::new(t));
                }
                if let Ok(t) = uid_trash {
                    return Some(Box::new(t));
                }

                None
            })
            .filter_map(|x| x)
            .collect::<Vec<_>>();
        trashes.insert(0, Box::new(home_trash));

        // Sort trashes by their priority such that admin trashes will always be before uid trashes
        trashes.sort_by_key(|x| x.priority() * -1);

        Ok(Self { trashes })
    }

    pub fn list(&self) -> impl Iterator<Item = PathBuf> {
        self.trashes
            .iter()
            .map(|x| x.list().unwrap())
            // This chains all of the individual iterators together
            .fold::<Box<dyn Iterator<Item = PathBuf>>, _>(
                Box::new(std::iter::empty()),
                |state: Box<dyn Iterator<Item = PathBuf>>,
                 x: fs::ReadDir|
                 -> Box<dyn Iterator<Item = PathBuf>> {
                    Box::new(state.chain(x.map(|x| x.unwrap().path())))
                },
            )
    }
}

fn list_mounts() -> crate::Result<Vec<PathBuf>> {
    fs::read("/proc/mounts")?
        .split(|x| *x as char == '\n')
        .filter(|x| !x.is_empty())
        .map(|x| x.split(|x| *x == b' ').nth(1))
        .map(|x| x.map(OsStr::from_bytes))
        .map(|x| x.map(PathBuf::from).ok_or(crate::Error::InvalidProcMounts))
        .collect()
}
