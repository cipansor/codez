//! Port of `modules/util/io.go`.

use std::io::{self, Read};

/// `util.NopCloser` — a reader that closes as a no-op.
///
/// Go needs this to satisfy `io.ReadCloser` over a bare writer; Rust's
/// `Read`/`Write` split makes the wrapper unnecessary, so only the semantics of
/// `ReadAtMost` and friends are ported.
///
/// `util.ReadAtMost` — reads at most `buf.len()` bytes, treating EOF as success.
///
/// `n` is less than `buf.len()` only when the reader runs out of data. The
/// Go version distinguishes "read fewer bytes" from "failed", and so does this.
pub fn read_at_most<R: Read>(r: &mut R, buf: &mut [u8]) -> io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        match r.read(&mut buf[n..]) {
            Ok(0) => break,
            Ok(read) => n += read,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(n)
}

/// `util.ReadWithLimit` — reads at most `n` bytes.
pub fn read_with_limit<R: Read>(r: &mut R, n: usize) -> io::Result<Vec<u8>> {
    read_with_limit_batched(r, 4 * 1024, n)
}

/// `util.readWithLimit` — the batching core of [`read_with_limit`].
fn read_with_limit_batched<R: Read>(r: &mut R, batch: usize, limit: usize) -> io::Result<Vec<u8>> {
    if limit <= batch {
        let mut buf = vec![0u8; limit];
        let n = read_at_most(r, &mut buf)?;
        buf.truncate(n);
        return Ok(buf);
    }

    let mut res: Vec<u8> = Vec::with_capacity(batch);
    let mut eof = false;
    while res.len() < limit && !eof {
        let want = batch.min(limit - res.len());
        let mut chunk = vec![0u8; want];
        let n = read_at_most(r, &mut chunk)?;
        // A short read means the reader is exhausted.
        eof = n != want;
        res.extend_from_slice(&chunk[..n]);
    }
    Ok(res)
}

/// `util.ErrNotEmpty` — reported by [`is_empty_reader`] for a non-empty reader.
pub const ERR_NOT_EMPTY: &str = "not-empty";

/// `util.IsEmptyReader` — consumes the reader and verifies it yields nothing.
pub fn is_empty_reader<R: Read>(r: &mut R) -> io::Result<()> {
    let mut buf = [0u8; 1];
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(_) => return Err(io::Error::other(ERR_NOT_EMPTY)),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Mirrors `readerWithError` in modules/util/io_test.go: yields its buffer
    /// until fewer than 2 bytes remain, then fails.
    struct ReaderWithError {
        buf: Cursor<Vec<u8>>,
    }

    impl Read for ReaderWithError {
        fn read(&mut self, p: &mut [u8]) -> io::Result<usize> {
            if self.buf.get_ref().len() - (self.buf.position() as usize) < 2 {
                return Err(io::Error::other("test error"));
            }
            self.buf.read(p)
        }
    }

    fn reader_with_error(bs: &[u8]) -> ReaderWithError {
        ReaderWithError {
            buf: Cursor::new(bs.to_vec()),
        }
    }

    // Transcribed from `TestReadWithLimit` in modules/util/io_test.go.
    #[test]
    fn read_with_limit_matches_upstream() {
        let bs: &[u8] = b"0123456789abcdef";

        let cases: &[(usize, usize, &[u8])] = &[
            (5, 2, b"01"),
            (5, 5, b"01234"),
            (5, 6, b"012345"),
            (5, bs.len(), bs),
            (5, 100, bs),
        ];
        for (batch, limit, want) in cases {
            let got = read_with_limit_batched(&mut Cursor::new(bs), *batch, *limit).unwrap();
            assert_eq!(got, *want, "batch {batch} limit {limit}");
        }

        // A reader that errors after yielding 10 bytes: a partial read up to the
        // error is returned, but asking for more surfaces the error.
        let got = read_with_limit_batched(&mut reader_with_error(bs), 5, 10).unwrap();
        assert_eq!(got, b"0123456789");

        let err = read_with_limit_batched(&mut reader_with_error(bs), 5, 100);
        assert!(err.is_err(), "expected the reader error to surface");
        assert!(err.unwrap_err().to_string().contains("test error"));

        // The public wrapper.
        assert_eq!(read_with_limit(&mut Cursor::new(bs), 2).unwrap(), b"01");
        assert_eq!(
            read_with_limit(&mut Cursor::new(bs), 9_999_999).unwrap(),
            bs
        );
    }

    #[test]
    fn is_empty_reader_detects_content() {
        assert!(is_empty_reader(&mut Cursor::new(Vec::new())).is_ok());
        let err = is_empty_reader(&mut Cursor::new(b"x")).unwrap_err();
        assert_eq!(err.to_string(), ERR_NOT_EMPTY);
    }
}
