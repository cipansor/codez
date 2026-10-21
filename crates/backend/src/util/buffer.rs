//! Port of `modules/util/buffer.go`.

/// `util.BufioScannerSplit` — a splitter that breaks input on a single byte.
///
/// Go returns a `bufio.SplitFunc` closure; in Rust the equivalent shape is an
/// iterator over the delimited tokens, which callers consume directly.
pub struct ByteDelimiterSplit<'a> {
    data: &'a [u8],
    delim: u8,
    pos: usize,
    done: bool,
}

impl<'a> ByteDelimiterSplit<'a> {
    /// Builds a splitter over `data`, breaking on `delim`.
    pub fn new(data: &'a [u8], delim: u8) -> Self {
        Self {
            data,
            delim,
            pos: 0,
            done: false,
        }
    }
}

impl<'a> Iterator for ByteDelimiterSplit<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.pos >= self.data.len() {
            return None;
        }
        match self.data[self.pos..].iter().position(|&b| b == self.delim) {
            Some(rel) => {
                let token = &self.data[self.pos..self.pos + rel];
                self.pos += rel + 1;
                Some(token)
            }
            None => {
                // Trailing data with no delimiter is the final token.
                self.done = true;
                Some(&self.data[self.pos..])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(data: &[u8], delim: u8) -> Vec<Vec<u8>> {
        ByteDelimiterSplit::new(data, delim)
            .map(|t| t.to_vec())
            .collect()
    }

    #[test]
    fn splits_on_the_delimiter() {
        assert_eq!(
            split(b"a\nb\nc", b'\n'),
            vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]
        );
    }

    #[test]
    fn keeps_empty_tokens_between_consecutive_delimiters() {
        assert_eq!(
            split(b"a\n\nb", b'\n'),
            vec![b"a".to_vec(), b"".to_vec(), b"b".to_vec()]
        );
    }

    #[test]
    fn trailing_delimiter_does_not_emit_a_final_empty_token() {
        // Matches bufio.ScanLines: "a\n" is one line, not two.
        assert_eq!(split(b"a\n", b'\n'), vec![b"a".to_vec()]);
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert_eq!(split(b"", b'\n'), Vec::<Vec<u8>>::new());
    }

    #[test]
    fn input_without_delimiter_is_one_token() {
        assert_eq!(split(b"abc", b'\n'), vec![b"abc".to_vec()]);
    }
}
