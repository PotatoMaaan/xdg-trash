use crate::{lexical_absolute, trash::Trash, trash_file::TrashFile, trashinfo::TrashInfo};
use chrono::Local;
use std::{
    borrow::Cow,
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::Path,
    rc::Rc,
};

impl Trash {
    /// Puts the file at `input_path` into this trashcan, returning the created trashfile
    pub fn put(self: Rc<Self>, input_path: &Path) -> crate::Result<TrashFile> {
        put_inner(self, input_path)
            .map_err(|e| crate::Error::FailedToTrashFile(input_path.to_owned(), Box::new(e)))
    }
}

fn put_inner(trash: Rc<Trash>, input_path: &Path) -> crate::Result<TrashFile> {
    let input_path = lexical_absolute(input_path)?;
    let input_path_meta = fs::symlink_metadata(&input_path)?;
    if input_path_meta.dev() != trash.device {
        return Err(crate::Error::DifferentDevice);
    }

    let trash_name = input_path.file_name().ok_or(crate::Error::HasNoFilename)?;

    let mut iter: u64 = 0;
    let (trashinfo, trash_name) = loop {
        iter += 1;
        let trash_name = if iter == 1 {
            Cow::Borrowed(trash_name)
        } else {
            let new_path = Path::new(&trash_name);

            /*
            If the base name is already in use (iter != 1) we append _x to the name,
            where x is the current iteration number. If the filename consists of a
            stem and an extension, we can construct a name like so: stem_x.ext
            This is nice because the extension is preserved properly.
            */
            let ext_preverving_name = if let Some(ext) = new_path.extension() {
                if let Some(stem) = new_path.file_stem() {
                    let mut name = stem.to_owned();
                    name.push("_");
                    name.push(iter.to_string());
                    name.push(".");
                    name.push(ext);
                    Some(name)
                } else {
                    None
                }
            } else {
                None
            };

            // If we can't build an extension preserving name, we just append the iteration number.
            Cow::Owned(ext_preverving_name.unwrap_or_else(|| {
                log::trace!("name without ext");
                let mut new = trash_name.to_owned();
                new.push("_");
                new.push(iter.to_string());
                new
            }))
        };

        log::trace!("Got name {:?}", trash_name);

        let mut trash_name_info = trash_name.clone().into_owned();
        trash_name_info.push(".trashinfo");
        let full_trash_path_info = trash.info_dir.join(trash_name_info);
        let trashinfo = {
            let trashinfo_file = match File::options()
                .create_new(true)
                .write(true)
                .truncate(true)
                .open(&full_trash_path_info)
            {
                Ok(v) => v,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::AlreadyExists => {
                        log::trace!("Name not unique, trying next..");
                        continue;
                    }

                    _ => {
                        return Err(crate::Error::IoError(e));
                    }
                },
            };

            let trashinfo = TrashInfo {
                path: if trash.use_relative_path {
                    input_path
                        .strip_prefix(&trash.mount_root)
                        .map_err(|_| crate::Error::InputNotChildOfTrashMount)
                        .map(|x| x.to_owned())?
                } else {
                    input_path.to_owned()
                },
                deleted_at: Local::now().naive_local(),
            };
            trashinfo.write_to(trashinfo_file)?;
            trashinfo
        };

        let full_trash_path_files = trash.files_dir.join(&trash_name);
        if let Err(e) = fs::rename(&input_path, full_trash_path_files) {
            log::error!("Failed to move file into trash, reverting trashinfo file");
            if fs::remove_file(full_trash_path_info).is_err() {
                log::error!("Failed to revert trashinfo file");
            }
            return Err(crate::Error::FailedToMoveFile(e));
        };

        break (trashinfo, trash_name);
    };

    Ok(TrashFile::new_unchecked(
        trash,
        trashinfo,
        trash_name.into_owned(),
    ))
}
