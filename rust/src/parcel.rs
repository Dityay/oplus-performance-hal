// A little-endian byte reader/writer for the AIDL parcel format as it appears
// on the wire.
//
// This exists so the parcelables can be round-tripped on the HOST, in plain
// Rust, without libbinder. The binder-facing path in wire.rs writes the same
// structures through AParcel; the two must agree, and the host tests pin the
// layout so a change on either side shows up here first.
//
// Format, as libbinder emits it:
//   i32      -> 4 bytes LE
//   bool     -> 4 bytes LE, 0 or 1
//   byte     -> 1 byte, then 3 bytes of padding to the next 4-byte boundary
//   String   -> i32 length in bytes (-1 for null), the UTF-8 bytes, a NUL, then
//               padding to 4
//   arrays   -> i32 count, then the elements, then padding to 4

#[derive(Default, Clone)]
pub struct Parcel {
    bytes: Vec<u8>,
    pos: usize,
}

impl Parcel {
    pub fn new() -> Self {
        Parcel {
            bytes: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Parcel { bytes, pos: 0 }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    #[cfg(test)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bytes left unread. Used to sanity-check length prefixes.
    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    /// Move the read cursor. A versioned parcelable declares its own end, and
    /// skipping to it keeps an older parser compatible with a newer body.
    pub fn seek(&mut self, at: usize) -> Option<()> {
        if at > self.bytes.len() {
            return None;
        }
        self.pos = at;
        Some(())
    }

    /// Current read cursor, used to measure a versioned parcelable's end.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Current write cursor, used to measure a versioned parcelable's size.
    pub fn write_pos(&self) -> usize {
        self.bytes.len()
    }

    /// Overwrite an i32 already written at `at`. AIDL parcelables lead with a
    /// size field that is only known once the fields are on the wire.
    pub fn patch_i32_at(&mut self, at: usize, value: i32) {
        self.bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_i32(if value { 1 } else { 0 });
    }

    /// `char` in AIDL is a single byte; libbinder pads back to a 4-byte
    /// boundary afterwards.
    pub fn write_byte(&mut self, value: u8) {
        self.bytes.push(value);
        self.pad();
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_i32(value.len() as i32);
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
        self.pad();
    }

    fn pad(&mut self) {
        while self.bytes.len() % 4 != 0 {
            self.bytes.push(0);
        }
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        if self.remaining() < 4 {
            return None;
        }
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.bytes[self.pos..self.pos + 4]);
        self.pos += 4;
        Some(i32::from_le_bytes(buf))
    }

    pub fn read_bool(&mut self) -> Option<bool> {
        self.read_i32().map(|v| v != 0)
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.remaining() < 1 {
            return None;
        }
        let value = self.bytes[self.pos];
        self.pos += 1;
        self.align();
        Some(value)
    }

    pub fn read_string(&mut self) -> Option<String> {
        let length = self.read_i32()?;
        if length < 0 {
            return Some(String::new());
        }
        let length = length as usize;
        if self.remaining() < length + 1 {
            return None;
        }
        let slice = &self.bytes[self.pos..self.pos + length];
        let text = String::from_utf8_lossy(slice).into_owned();
        self.pos += length + 1; // skip the NUL
        self.align();
        Some(text)
    }

    fn align(&mut self) {
        while self.pos % 4 != 0 {
            self.pos += 1;
        }
        if self.pos > self.bytes.len() {
            self.pos = self.bytes.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_are_four_little_endian_bytes() {
        let mut p = Parcel::new();
        p.write_i32(1);
        assert_eq!(p.as_bytes(), &[1, 0, 0, 0]);
    }

    #[test]
    fn negative_integers_survive() {
        let mut p = Parcel::new();
        p.write_i32(-2);
        assert_eq!(p.read_i32(), Some(-2));
    }

    #[test]
    fn a_string_is_length_then_bytes_then_nul_then_padding() {
        let mut p = Parcel::new();
        p.write_string("ab");
        assert_eq!(p.as_bytes(), &[2, 0, 0, 0, b'a', b'b', 0, 0]);
        assert_eq!(p.as_bytes().len() % 4, 0);
    }

    #[test]
    fn a_string_round_trips() {
        let mut p = Parcel::new();
        p.write_string("com.example.app");
        assert_eq!(p.read_string().unwrap(), "com.example.app");
    }

    #[test]
    fn a_byte_takes_one_byte_plus_padding() {
        let mut p = Parcel::new();
        p.write_byte(0xFF);
        assert_eq!(p.as_bytes(), &[0xFF, 0, 0, 0]);
        assert_eq!(p.read_byte(), Some(0xFF));
    }

    #[test]
    fn reads_past_the_end_are_none_not_panics() {
        let mut p = Parcel::new();
        p.write_i32(1);
        assert_eq!(p.read_i32(), Some(1));
        assert_eq!(p.read_i32(), None);
    }

    #[test]
    fn truncated_string_is_none() {
        let mut p = Parcel::new();
        p.write_i32(100);
        assert_eq!(p.read_string(), None);
    }
}