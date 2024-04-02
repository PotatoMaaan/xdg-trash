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
    FailedToFindHomeTrash(#[source] Box<Self>),

    /** The trash at {0} is invalid because the sticky bit not set */
    NotSticky(PathBuf),

    /** The trash at {0} is invalid because it is a symlink */
    IsSymlink(PathBuf),

    /** The /proc/mounts file was in an unexpected format */
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
}

pub type Result<T> = core::result::Result<T, Error>;
