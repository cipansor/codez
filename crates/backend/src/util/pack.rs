//! Port of `modules/util/pack.go`.
//!
//! Go uses `encoding/gob`, a self-describing stream format. Rust has no
//! equivalent in std, so this is a minimal length-prefixed, type-tagged format
//! that reproduces the observable behaviour the upstream tests rely on: values
//! are written and read in sequence, and reading a value into a mismatched type
//! is an error rather than a silent reinterpretation.
//!
//! It is deliberately NOT wire-compatible with gob. Codez never exchanges
//! packed data with Gitea, so the only requirement is that a pack round-trips
//! through this module.

use std::fmt;

/// Tag bytes identifying the concrete type of each packed value.
const TAG_STRING: u8 = 1;
const TAG_I64: u8 = 2;
const TAG_F32: u8 = 3;

/// `util.PackData`/`util.UnpackData` value contract.
///
/// Upstream relies on gob's runtime type information; here the type is carried
/// explicitly so a decode into the wrong type can fail the same way.
pub trait Packable: Sized {
    fn tag() -> u8;
    fn encode(&self, out: &mut Vec<u8>);
    fn decode(data: &[u8]) -> Result<Self, PackError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PackError {
    /// The buffer ended in the middle of a value.
    Truncated,
    /// The value on the wire is not the type being decoded into.
    TypeMismatch { expected: u8, found: u8 },
    /// No more values to read.
    Empty,
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::Truncated => write!(f, "packed data is truncated"),
            PackError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected tag {expected}, found {found}")
            }
            PackError::Empty => write!(f, "no more values to unpack"),
        }
    }
}

impl std::error::Error for PackError {}

impl Packable for String {
    fn tag() -> u8 {
        TAG_STRING
    }
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.len() as u32).to_le_bytes());
        out.extend_from_slice(self.as_bytes());
    }
    fn decode(data: &[u8]) -> Result<Self, PackError> {
        let len = u32::from_le_bytes(
            data.get(..4)
                .ok_or(PackError::Truncated)?
                .try_into()
                .unwrap(),
        ) as usize;
        let bytes = data.get(4..4 + len).ok_or(PackError::Truncated)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| PackError::Truncated)
    }
}

impl Packable for i64 {
    fn tag() -> u8 {
        TAG_I64
    }
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }
    fn decode(data: &[u8]) -> Result<Self, PackError> {
        Ok(i64::from_le_bytes(
            data.get(..8)
                .ok_or(PackError::Truncated)?
                .try_into()
                .unwrap(),
        ))
    }
}

impl Packable for f32 {
    fn tag() -> u8 {
        TAG_F32
    }
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.to_le_bytes());
    }
    fn decode(data: &[u8]) -> Result<Self, PackError> {
        Ok(f32::from_le_bytes(
            data.get(..4)
                .ok_or(PackError::Truncated)?
                .try_into()
                .unwrap(),
        ))
    }
}

/// `util.PackData` — encodes the given values in sequence.
pub fn pack_data(values: &[&dyn PackableRef]) -> Vec<u8> {
    let mut out = Vec::new();
    for v in values {
        out.push(v.tag());
        v.encode(&mut out);
    }
    out
}

/// Object-safe shim over [`Packable`] so heterogeneous values can be packed.
pub trait PackableRef {
    fn tag(&self) -> u8;
    fn encode(&self, out: &mut Vec<u8>);
}

impl<T: Packable> PackableRef for T {
    fn tag(&self) -> u8 {
        T::tag()
    }
    fn encode(&self, out: &mut Vec<u8>) {
        Packable::encode(self, out)
    }
}

/// Cursor over packed bytes, decoding values in the order they were written.
pub struct Unpacker<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Unpacker<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Decodes the next value as `T`, failing when the tags disagree.
    pub fn read_next<T: Packable>(&mut self) -> Result<T, PackError> {
        let tag = *self.data.get(self.pos).ok_or(PackError::Empty)?;
        self.pos += 1;
        if tag != T::tag() {
            return Err(PackError::TypeMismatch {
                expected: T::tag(),
                found: tag,
            });
        }
        // The payload length is type-dependent, so let the type consume a slice
        // of the remaining data and report how much it used.
        let rest = &self.data[self.pos..];
        let value = T::decode(rest)?;
        self.pos += consumed_len(rest, tag)?;
        Ok(value)
    }
}

/// Bytes a value of `tag` occupies at the start of `data`.
fn consumed_len(data: &[u8], tag: u8) -> Result<usize, PackError> {
    match tag {
        TAG_STRING => {
            let len = u32::from_le_bytes(
                data.get(..4)
                    .ok_or(PackError::Truncated)?
                    .try_into()
                    .unwrap(),
            ) as usize;
            4usize.checked_add(len).ok_or(PackError::Truncated)
        }
        TAG_I64 => Ok(8),
        TAG_F32 => Ok(4),
        _ => Err(PackError::TypeMismatch {
            expected: 0,
            found: tag,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Transcribed from `TestPackAndUnpackData` in modules/util/pack_test.go.
    #[test]
    fn pack_and_unpack_round_trips_in_sequence() {
        let s = "string".to_string();
        let i: i64 = 4;
        let f: f32 = 4.1;

        let data = pack_data(&[&s, &i, &f]);

        // All three values, in order.
        let mut u = Unpacker::new(&data);
        assert_eq!(u.read_next::<String>().unwrap(), "string");
        assert_eq!(u.read_next::<i64>().unwrap(), 4);
        assert_eq!(u.read_next::<f32>().unwrap(), 4.1);

        // Reading only the first value succeeds.
        let mut u = Unpacker::new(&data);
        assert_eq!(u.read_next::<String>().unwrap(), "string");

        // Reading the first value as the wrong type fails.
        let mut u = Unpacker::new(&data);
        assert_eq!(
            u.read_next::<i64>().unwrap_err(),
            PackError::TypeMismatch {
                expected: TAG_I64,
                found: TAG_STRING
            }
        );

        // Reading two values where the second is the wrong type fails.
        let mut u = Unpacker::new(&data);
        assert_eq!(u.read_next::<String>().unwrap(), "string");
        assert!(matches!(
            u.read_next::<f32>(),
            Err(PackError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn unpacking_past_the_end_is_empty() {
        let data = pack_data(&[&"only".to_string()]);
        let mut u = Unpacker::new(&data);
        assert_eq!(u.read_next::<String>().unwrap(), "only");
        assert_eq!(u.read_next::<String>().unwrap_err(), PackError::Empty);
    }

    #[test]
    fn truncated_payload_is_reported() {
        let data = pack_data(&[&"abcd".to_string()]);
        let mut u = Unpacker::new(&data[..5]);
        assert_eq!(u.read_next::<String>().unwrap_err(), PackError::Truncated);
    }
}
