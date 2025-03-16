use crate::rpc::decode::Decode;

use super::{compactstring::CompactValueParseError, decode_varint, Offset};

pub struct CompactArray<T>
where
    T: Decode<T> + Offset,
{
    pub elements: Vec<T>,
}

#[doc(hidden)]
impl<T> CompactArray<T>
where
    T: Decode<T> + Offset,
{
    pub fn new(buf: &[u8]) -> Result<Self, CompactValueParseError> {
        let (length, size) = decode_varint(buf)?;
        let mut elements: Vec<T> = Vec::new();
        let mut ptr = size;

        for _ in 0..length {
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

        Ok(CompactArray { elements })
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

        let compact_array = CompactArray::<CompactString>::new(&buf[..]).unwrap();

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
