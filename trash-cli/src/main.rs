use clap::Parser;
use sha2::{Digest, Sha256};
use std::{
    env, fmt::Write, io::stdin, os::unix::ffi::OsStrExt, path::Path, process::ExitCode,
    str::FromStr,
};
use xdg_trash::TrashFile;

mod cli;
mod commands;
mod streaming_table;
mod user_input;

fn main() -> ExitCode {
    microlog::init(microlog::LevelFilter::Info);

    let bin_name = env::args().next();
    let bin_name = bin_name
        .as_ref()
        .and_then(|x| {
            let p = Path::new(x);
            p.file_name().map(|x| x.to_str())
        })
        .flatten();

    let result = match bin_name {
        Some("trash") => {
            let args = cli::PutArgs::parse();
            commands::put(args)
        }
        Some("trash-put") => {
            let args = cli::PutArgs::parse();
            commands::put(args)
        }
        Some("trash-list") => {
            let args = cli::ListArgs::parse();
            commands::list(args)
        }
        Some("trash-empty") => {
            let args = cli::EmptyArgs::parse();
            commands::empty(args)
        }
        Some("trash-restore") => {
            let args = cli::RestoreArgs::parse();
            commands::restore(args)
        }
        Some("trash-rm") => {
            let args = cli::RemoveArgs::parse();
            commands::remove(args)
        }
        _ => {
            log::debug!("Not called with known filename, acting as root command");
            let root_args = cli::RootArgs::parse();
            match root_args.subcommand {
                cli::SubCmd::Put(args) => commands::put(args),
                cli::SubCmd::List(args) => commands::list(args),
                cli::SubCmd::Empty(args) => commands::empty(args),
                cli::SubCmd::Restore(args) => commands::restore(args),
                cli::SubCmd::Remove(args) => commands::remove(args),
                cli::SubCmd::ListTrashes(args) => commands::list_trashes(args),
                cli::SubCmd::Fix(args) => commands::fix(args),
            }
        }
    };

    if let Err(e) = result {
        log::error!("{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

pub const ID_LEN: usize = 10;

pub trait HashID {
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
        encode_hex(hash).chars().take(ID_LEN).collect()
    }
}

impl HashID for TrashFile {
    fn file_bytes(&self) -> Vec<u8> {
        self.original_path().as_os_str().as_bytes().to_vec()
    }
}
