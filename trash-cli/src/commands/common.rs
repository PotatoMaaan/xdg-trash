use crate::{streaming_table::StreamingTable, HashID};
use anyhow::Context;
use std::{
    io::{stdin, stdout, Write},
    path::Path,
    str::FromStr,
};
use xdg_trash::{TrashFile, UnifiedTrash};

pub fn choose(mut options: Vec<TrashFile>) -> TrashFile {
    if options.len() == 1 {
        return options.remove(0);
    }

    println!("Multiple items match:\n");

    let table = StreamingTable::draw_header([
        ("Index", Some(5)),
        ("Deleted at", Some(19)),
        ("Original path", None),
    ]);
    for (i, option) in options.iter().enumerate() {
        table.draw_row([
            &(i + 1).to_string(),
            &option.deleted_at().to_string(),
            option.original_path().to_string_lossy().as_ref(),
        ]);
    }
    println!();

    loop {
        print!("Choose one [{:?}]: ", 1..options.len());
        stdout().flush().unwrap();
        let Some(choice) = read_generic::<usize>() else {
            log::error!("Input is not a valid number.\n");
            continue;
        };

        let final_index = choice.wrapping_sub(1);
        match options.get(final_index) {
            Some(_) => break options.remove(final_index),
            None => {
                log::error!("Number out of bounds, please pick a number in the range.\n");
                continue;
            }
        }
    }
}

pub fn ask_yes_no(prompt: &str, default: bool) -> bool {
    loop {
        print!("{}: [{}] ", prompt, if default { "Y/n" } else { "y/N" });
        stdout().flush().unwrap();
        let Some(input) = read_line() else {
            return default;
        };
        if input.is_empty() {
            return default;
        }
        let input = input.to_ascii_lowercase();

        match input.as_str() {
            "y" => break true,
            "n" => break false,
            _ => continue,
        }
    }
}

pub fn read_line() -> Option<String> {
    stdin().lines().next().and_then(Result::ok)
}

pub fn read_generic<T: FromStr>() -> Option<T> {
    read_line().and_then(|x| x.parse().ok())
}

pub fn list_trashes_matching_status(id_or_path: &str) -> anyhow::Result<Vec<TrashFile>> {
    let trash = UnifiedTrash::new().context("Failed to init trash")?;
    print!("Listing files, this might take a moment.");
    stdout().flush().unwrap();

    let matches = trash
        .list()
        .inspect(|x| {
            log::debug!("Listing: {x:#?}");
            print!(".");
            stdout().flush().unwrap();
        })
        .filter_map(Result::ok)
        .filter(|x| x.id() == id_or_path || x.original_path() == Path::new(&id_or_path))
        .collect::<Vec<_>>();
    println!();
    println!();

    Ok(matches)
}
