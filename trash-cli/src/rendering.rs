pub struct StreamingTable<const N: usize> {}

impl<const N: usize> StreamingTable<N> {
    pub fn draw_header(header: [(&str, Option<usize>); N]) -> StreamingTable<N> {
        for (i, (name, len)) in header.into_iter().enumerate() {
            if i != 0 {
                print!(" ");
            }

            if let Some(pad_len) = len.map(|len| len - name.len()) {
                print!("{}", name);
                print!("{}", " ".repeat(pad_len));
            } else {
                print!("{}", name);
            }

            if (i + 1) != N {
                print!(" |");
            }
        }
        println!();

        Self {}
    }

    pub fn draw_row(&self, data: [&str; N]) {
        for (i, col) in data.into_iter().enumerate() {}
    }
}
