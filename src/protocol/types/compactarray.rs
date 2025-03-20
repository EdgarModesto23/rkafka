use std::fmt::Debug;

use bytes::BufMut;

use crate::rpc::{decode::Decode, encode::Encode};

use super::{compactstring::CompactValueParseError, decode_varint, Offset};

pub struct CompactArray<T>
where
    T: Decode<T> + Offset,
{
    pub elements: Vec<T>,
}

impl<T> Debug for CompactArray<T>
where
    T: Decode<T> + Offset + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompactArray")
            .field("elements", &self.elements) // Use Debug on Vec<T>
            .finish()
    }
}

#[doc(hidden)]
impl<T> CompactArray<T>
where
    T: Decode<T> + Offset,
{
    pub fn new(buf: &[u8]) -> Result<(Self, usize), CompactValueParseError> {
        let (length, size) = decode_varint(buf)?;
        println!("{length:?}");
        let mut elements: Vec<T> = Vec::new();
        let mut ptr = size;

        for _ in 0..length - 1 {
            if ptr >= buf.len() {
                break;
            }

            let curr = &buf[ptr..];
            if let Ok(decoded) = T::decode(curr) {
                ptr += decoded.get_offset() as usize;
                elements.push(decoded);
            } else {
                return Err(CompactValueParseError::InvalidVarint);
            }
        }

        Ok((CompactArray { elements }, ptr))
    }
}

impl<T> Encode for CompactArray<T>
where
    T: Decode<T> + Offset,
{
    fn encode(&self, buf: &mut bytes::BytesMut) {
        if self.elements.len() < 1 {
            buf.put_u8(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::types::compactstring::CompactString;

    use super::*;

    #[test]
    fn test_compact_array_decoding_valid() {
        // Test case where the buffer is correctly decoded
        let buf: Vec<u8> = vec![
            2, // length of elements (2 elements)
            5, b'H', b'e', b'l', b'l', b'o', // first CompactString: "Hello"
            3, b'B', b'y', b'e', // second CompactString: "Bye"
        ];

        let (compact_array, _) = CompactArray::<CompactString>::new(&buf[..]).unwrap();

        assert_eq!(compact_array.elements.len(), 2);
        assert_eq!(compact_array.elements[0].value, "Hello");
        assert_eq!(compact_array.elements[1].value, "Bye");
    }

    #[test]
    fn test_compact_array_decoding_invalid_varint() {
        // Test case where the buffer is invalid (not enough data for the expected length)
        let buf: Vec<u8> = vec![
            2, // length of elements (2 elements)
            5, b'H', b'e', b'l', b'l', // missing byte for "Hello"
        ];

        let result = CompactArray::<CompactString>::new(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_compact_array_empty_buffer() {
        // Test case where the buffer is empty
        let buf: Vec<u8> = vec![];

        let result = CompactArray::<CompactString>::new(&buf);
        assert!(result.is_err());
    }
}
