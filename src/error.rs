use displaydoc::Display;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum Error {
    /** The HOME env var was not set */
    Homeless,

    /** Io Error: {0} */
    IoError(#[from] io::Error),

    /** Failed to determine the home trash */
    FailedToFindHomeTrash(#[source] Box<Self>),

    /** The trash at {0} is invalid because the sticky bit not set */
    NotSticky(PathBuf),

    /** The trash at {0} is invalid because it is a symlink */
    IsSymlink(PathBuf),

    /** The /proc/mounts file was not found or in an unexpected format */
    InvalidProcMounts,

    /** The first line was invalid */
    InvalidFirstLine,

    /** The key/value pairs were invalid */
    InvalidKeyValues,

    /** The key {0} was not found */
    MissingKey(&'static str),

    /** The datetime was invalid: {0} */
    InvalidDateTime(#[from] chrono::ParseError),

    /** None of the available parsers matched the datetime: [errors:?] */
    InvalidDateTimeNoParserMatched { errors: Vec<chrono::ParseError> },

    /** The trashinfo file at {0} is invalid */
    InvalidTrashinfoFile(PathBuf, #[source] Box<Self>),

    /** The file {0} does not have a file stem even though it should */
    HasNoFileStem(PathBuf),

    /** The trashinfo file has no corresponding actual file */
    OrphanedTrashinfoFile,

    /** The trashinfo file did not have a .trashinfo extension */
    InvalidTrashinfoExt,

    /** Can only trash file from the same physical device */
    DifferentDevice,

    /** Failed to trash file {0}: {1} */
    FailedToTrashFile(PathBuf, #[source] Box<Self>),

    /** The path has no filename */
    HasNoFilename,

    /**
     * The file was not a child component of the trash mount root.
     * This ususually happens when multiple qualifying trash dirs exist on one physical device
     */
    InputNotChildOfTrashMount,

    /** Failed to move file: {0} */
    FailedToMoveFile(std::io::Error),

    /** Failed to create a new trashcan at {0} */
    FailedToCreateTrash(PathBuf, #[source] Box<Self>),

    /** No appropriate trash can was found */
    NoTrashFound,
}

/// A Result type predefined with the libraries error
pub type Result<T> = core::result::Result<T, Error>;
