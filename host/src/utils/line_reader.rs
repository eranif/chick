pub struct LineReader<'a> {
    data: &'a [u8],
}

impl<'a> LineReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for LineReader<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let newline_pos = self.data.iter().position(|&b| b == b'\n');

        let (line, rest) = match newline_pos {
            Some(pos) => {
                let line = &self.data[..pos];
                let rest = &self.data[pos + 1..];
                (line, rest)
            }
            None => {
                let line = self.data;
                let rest: &[u8] = &[];
                (line, rest)
            }
        };

        self.data = rest;
        core::str::from_utf8(line).ok().map(|s| s.trim_end_matches('\r'))
    }
}