use std::fmt::Debug;

use crate::rpc::decode::{Decode, DecodeError};

use super::{compactstring::CompactString, Offset};

pub struct TopicStr {
    pub value: CompactString,
    pub tag_buffer: u8,
    pub bytes_len: usize,
}

impl Debug for TopicStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TopicString")
            .field("value", &self.value.value)
            .finish()
    }
}

impl Offset for TopicStr {
    fn get_offset(&self) -> u64 {
        self.value.size_len_bytes + 1
    }
}

impl Decode<TopicStr> for TopicStr {
    fn decode(buf: &[u8]) -> Result<TopicStr, crate::rpc::decode::DecodeError> {
        let value = TopicStr::new(buf).map_err(|e| DecodeError::InvalidBuffer(format!("{e:?}")))?;
        Ok(value)
    }
}

impl TopicStr {
    fn new(buf: &[u8]) -> Result<TopicStr, anyhow::Error> {
        let mut value = CompactString::new(buf)?;
        value.value = value.value.trim_end_matches('\0').to_string();
        value.size -= 1;
        value.size_len_bytes -= 1;
        let tag_buffer = buf[(value.size_len_bytes) as usize];
        let bytes_len = (value.size_len_bytes) as usize;

        Ok(TopicStr {
            value,
            tag_buffer,
            bytes_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test case 1: Valid buffer
    #[test]
    fn test_valid_topic_str() {
        let buf: &[u8] = &[
            0x03, // Varint length (3 bytes)
            b'F', b'o', b'o', // UTF-8 bytes for "Foo"
            0x01, // A tag byte, for example
        ];

        let result = TopicStr::new(buf);

        assert!(result.is_ok());
        let topic_str = result.unwrap();
        assert_eq!(topic_str.value.value, "Foo");
        assert_eq!(topic_str.tag_buffer, 0x01);
        assert_eq!(topic_str.bytes_len, 5); // Length of 3 (Foo) + 1 (tag byte)
    }

    #[test]
    fn test_valid_topic_str_junk_after() {
        let buf: &[u8] = &[
            0x03, // Varint length (3 bytes)
            b'F', b'o', b'o', // UTF-8 bytes for "Foo"
            0x01, // A tag byte, for example
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = TopicStr::new(buf);

        assert!(result.is_ok());
        let topic_str = result.unwrap();
        assert_eq!(topic_str.value.value, "Foo");
        assert_eq!(topic_str.tag_buffer, 0x01);
        assert_eq!(topic_str.bytes_len, 5); // Length of 3 (Foo) + 1 (tag byte)
    }

    // Test case 2: Buffer too small (not enough bytes for the length prefix)
    #[test]
    fn test_buffer_too_small() {
        let buf: &[u8] = &[0x03]; // Only length prefix, no data

        let result = TopicStr::new(buf);

        assert!(result.is_err());
    }

    // Test case 3: Invalid UTF-8 string
    #[test]
    fn test_invalid_utf8() {
        // A valid length prefix (3), but invalid UTF-8 bytes
        let buf: &[u8] = &[0x03, 0xFF, 0xFF, 0xFF];

        let result = TopicStr::new(buf);

        assert!(result.is_err());
    }

    // Test case 4: Insufficient buffer for CompactString length and string data
    #[test]
    fn test_insufficient_buffer_for_string_data() {
        // Length is 5, but we only provide 4 bytes of buffer
        let buf: &[u8] = &[0x05, b'F', b'o', b'o', b'B'];

        let result = TopicStr::new(buf);

        assert!(result.is_err());
    }
}
