use bytes::BytesMut;
use compactstring::CompactValueParseError;

pub mod compactarray;
pub mod compactstring;
pub mod nullstring;
pub mod partition;
pub mod record;
pub mod topicstr;

pub trait Offset {
    fn get_offset(&self) -> u64;
}

#[doc(hidden)]
pub fn decode_varint(data: &[u8]) -> Result<(u64, usize), CompactValueParseError> {
    let mut value = 0u64;
    let mut shift = 0;
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];
        value |= u64::from(byte & 0x7F) << shift;
        shift += 7;
        i += 1;

        if byte & 0x80 == 0 {
            return Ok((value, i));
        }

        if shift >= 64 {
            return Err(CompactValueParseError::InvalidVarint);
        }
    }

    Err(CompactValueParseError::InvalidVarint)
}

pub fn encode_zigzag(value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    while value >= 0x80 {
        result.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }

    result.push(value as u8);
    result
}

pub trait CompactEncode {
    fn encode_compact(&self, buf: &mut BytesMut);
}

impl Offset for i32 {
    fn get_offset(&self) -> u64 {
        4
    }
}
