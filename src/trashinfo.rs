use chrono::NaiveDateTime;
use std::{
    collections::HashMap, ffi::OsStr, io::Write, os::unix::ffi::OsStrExt, path::PathBuf,
    str::FromStr,
};

#[derive(Debug)]
/// 1:1 representation of a .trashinfo file.
/// The caller is responsible for handling relative paths etc.
pub struct TrashInfo {
    pub path: PathBuf,
    pub deleted_at: NaiveDateTime,
}

const PATH_KEY: &str = "Path";
const DELDATE_KEY: &str = "DeletionDate";

impl FromStr for TrashInfo {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        if lines.next() != Some("[Trash Info]") {
            return Err(crate::Error::InvalidFirstLine);
        }

        fn parse_line(line: &str) -> Option<(&str, &str)> {
            let mut line = line.split('=');
            Some((line.next()?, line.next()?))
        }

        let kv_lookup = lines
            .map(parse_line)
            .collect::<Option<HashMap<_, _>>>()
            .ok_or(crate::Error::InvalidKeyValues)?;

        let path = kv_lookup
            .get(PATH_KEY)
            .ok_or(crate::Error::MissingKey(PATH_KEY))?;
        let path = urlencoding::decode_binary(path.as_bytes());
        let path = OsStr::from_bytes(&path);
        let path = PathBuf::from(path);

        let deleted_at = kv_lookup
            .get(DELDATE_KEY)
            .ok_or(crate::Error::MissingKey(DELDATE_KEY))?;

        let deleted_at = try_different_parsers(&deleted_at)
            .map_err(|e| crate::Error::InvalidDateTimeNoParserMatched { errors: e })?;

        Ok(Self { path, deleted_at })
    }
}

impl TrashInfo {
    fn create_trashinfofile(&self) -> String {
        let encoded = urlencoding::encode_binary(self.path.as_os_str().as_bytes());
        format!(
            "[Trash Info]\nPath={}\nDeletionDate={}",
            encoded,
            // The same format that nautilus and dolphin use. The spec claims rfc3339, but that doesn't work out at all...
            self.deleted_at.format("%Y-%m-%dT%H:%M:%S")
        )
    }

    pub fn write_to(&self, mut w: impl Write) -> crate::Result<()> {
        let file = self.create_trashinfofile();
        w.write_all(file.as_bytes())
            .map_err(|e| crate::Error::IoError(e))
    }
}

fn try_different_parsers(input: &str) -> Result<NaiveDateTime, Vec<chrono::ParseError>> {
    /// This covers most real-world cases
    fn parser1(input: &str) -> Result<NaiveDateTime, chrono::ParseError> {
        chrono::NaiveDateTime::from_str(input)
    }

    /// According to the spec, the datetime should be rfc3339, but i've not found a single real example that actually works here
    /// Even the provided sample time in the spec does not parse with this.
    fn parser2(input: &str) -> Result<NaiveDateTime, chrono::ParseError> {
        chrono::DateTime::parse_from_rfc3339(input).map(|x| x.naive_local())
    }

    /// This works for the example provided in the spec.
    fn parser3(input: &str) -> Result<NaiveDateTime, chrono::ParseError> {
        chrono::NaiveDateTime::parse_from_str(input, "%Y%m%dT%H:%M:%S")
    }

    /// Let's just also throw this in because why not
    fn parser4(input: &str) -> Result<NaiveDateTime, chrono::ParseError> {
        chrono::DateTime::parse_from_rfc2822(input).map(|x| x.naive_local())
    }

    let mut errs = vec![];
    for parser in [parser1, parser2, parser3, parser4] {
        let parse_res = parser(input);
        match parse_res {
            Ok(p) => {
                return Ok(p);
            }
            Err(e) => {
                errs.push(e);
                log::trace!("Datetime parser failed: {e}");
            }
        }
    }

    log::debug!("No datetime parsers matched");
    Err(errs)
}
