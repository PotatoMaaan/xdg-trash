use sha2::{Digest, Sha256};
use std::{fmt::Write, os::unix::ffi::OsStrExt};
use streaming_table::StreamingTable;
use xdg_trash::{TrashFile, UnifiedTrash};

mod cli;
mod streaming_table;

fn main() {
    microlog::init(microlog::LevelFilter::Info);
    let trash = UnifiedTrash::new().unwrap();

    let mut table = StreamingTable::draw_header([
        ("ID", Some(10)),
        ("Deleted at", Some(19)),
        ("Original Location", None),
    ]);

    let list = trash.list();

    let list: Box<dyn Iterator<Item = TrashFile>> = if false {
        let mut vec = list.collect::<Result<Vec<_>, _>>().unwrap();
        vec.sort_by_key(|x| x.deleted_at());
        Box::new(vec.into_iter())
    } else {
        Box::new(list.map(|x| x.unwrap()))
    };

    for file in list {
        table.draw_row([
            &file.id(),
            &file.deleted_at().to_string(),
            &file.original_path().to_string_lossy(),
        ]);
    }

    table.draw_seperator();
}

pub trait IDable {
    fn file_bytes(&self) -> Vec<u8>;

    fn id(&self) -> String {
        pub fn encode_hex(bytes: &[u8]) -> String {
            let mut s = String::with_capacity(bytes.len() * 2);
            for &b in bytes {
                write!(&mut s, "{:02x}", b).unwrap();
            }
            s
        }

        let hash = Sha256::digest(self.file_bytes());
        let hash = hash.as_slice();
        encode_hex(hash).chars().take(10).collect()
    }
}

impl IDable for TrashFile {
    fn file_bytes(&self) -> Vec<u8> {
        self.original_path().as_os_str().as_bytes().to_vec()
    }
}
