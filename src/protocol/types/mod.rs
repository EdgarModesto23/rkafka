use compactstring::CompactValueParseError;

pub mod compactarray;
pub mod compactstring;
pub mod nullstring;

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
