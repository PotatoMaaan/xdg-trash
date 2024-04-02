use std::{io, path::PathBuf};

use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /** The HOME env var was not set */
    Homeless,

    /** Io Error: {0} */
    IoError(#[from] io::Error),

    /** Failed to determine the home trash */
    FailedToFindHomeTrash(#[source] Box<Error>),

    /** The trash at {0} is invalid because the sticky bit not set */
    NotSticky(PathBuf),

    /** The trash at {0} is invalid because it is a symlink */
    IsSymlink(PathBuf),

    /** The /proc/mounts file was in an unexpected format */
    InvalidProcMounts,
}

pub type Result<T> = core::result::Result<T, Error>;
