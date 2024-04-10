use colored::Colorize;

pub struct StreamingTable<const COLS: usize> {
    padding: [Option<usize>; COLS],
    lengths: [usize; COLS],
}

impl<const COLS: usize> StreamingTable<COLS> {
    pub fn draw_header(header: [(&str, Option<usize>); COLS]) -> StreamingTable<COLS> {
        let mut lengths = [0; COLS];

        for (i, (name, len)) in header.into_iter().enumerate() {
            if i != 0 {
                print!(" ");
                lengths[i] += 1;
            }

            if let Some(pad_len) = len.map(|len| len.saturating_sub(name.len())) {
                print!("{}", name.white());
                lengths[i] += name.len();
                print!("{}", " ".repeat(pad_len));
                lengths[i] += pad_len;
            } else {
                print!("{}", name.white());
                lengths[i] += name.len();
            }

            if (i + 1) != COLS {
                print!("{}", " |".bright_black());
                lengths[i] += 2;
            }
        }
        println!();

        let table = Self {
            lengths,
            padding: header
                .into_iter()
                .map(|(_, pad)| pad)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        };

        table.draw_seperator();

        table
    }

    pub fn draw_seperator(&self) {
        for (i, len) in self.lengths.into_iter().enumerate() {
            print!("{}", "-".repeat(len.saturating_sub(1)).bright_black());

            if i + 1 != COLS {
                print!("{}", "+".bright_black());
            } else {
                if let Some(size_hint) = self.padding[i] {
                    print!("{}", "-".repeat(size_hint).bright_black());
                } else {
                    print!("{}", "-".repeat(5).bright_black());
                }
            }
        }
        println!();
    }

    pub fn draw_row(&self, data: [&str; COLS]) {
        for (i, col) in data.into_iter().enumerate() {
            if i != 0 {
                print!(" ");
            }

            if let Some(pad_len) = self.padding[i].map(|len| len.saturating_sub(col.len())) {
                print!("{}", col);
                print!("{}", " ".repeat(pad_len));
            } else {
                print!("{}", col);
            }

            if (i + 1) != COLS {
                print!("{}", " |".bright_black());
            }
        }
        println!();
    }
}
