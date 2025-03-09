pub struct CompactString {
    pub value: String,
    pub size: usize,
    pub size_len_bytes: u64,
}

use std::str;

use crate::rpc::decode::{Decode, DecodeError};

#[derive(Debug, PartialEq)]
pub enum CompactStringParseError {
    InvalidVarint,
    InvalidUtf8(str::Utf8Error),
    InvalidLengthPrefix,
}

fn decode_varint(data: &[u8]) -> Result<(u64, usize), CompactStringParseError> {
    let mut value = 0u64;
    let mut shift = 0;
    let mut i = 0;

    while i < data.len() {
        let byte = data[i];
        value |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        i += 1;

        if byte & 0x80 == 0 {
            return Ok((value, i));
        }

        if shift >= 64 {
            return Err(CompactStringParseError::InvalidVarint);
        }
    }

    Err(CompactStringParseError::InvalidVarint)
}

impl CompactString {
    /// Decodes a compact string from the given byte buffer.
    ///
    /// This function reads a varint-encoded length prefix from the buffer, followed by the string bytes in UTF-8 format.
    /// Unlike the ``CompactString::new``, this function returns the decoded string along with the total number of bytes read (length prefix and string data).
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing the encoded compact string. The buffer is expected to start with a varint encoding the length of the string, followed by the UTF-8 string itself.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple containing:
    /// - The decoded `String`
    /// - The total number of bytes read (including the length prefix and string).
    ///
    /// In case of an error:
    /// - `CompactStringParseError::InvalidLengthPrefix` if the length is invalid or exceeds the buffer size.
    /// - `CompactStringParseError::InvalidUtf8` if the string cannot be decoded as UTF-8.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The length encoded in the buffer is larger than the available bytes in the buffer.
    /// - The buffer does not contain a valid UTF-8 string.
    ///
    pub fn get(buf: &[u8]) -> Result<(String, u64), CompactStringParseError> {
        let (length, varint_bytes_read) = decode_varint(buf)?;

        if length > (buf.len() - varint_bytes_read) as u64 {
            return Err(CompactStringParseError::InvalidLengthPrefix);
        }

        let total_bytes_read = varint_bytes_read as u64 + length;

        let string_bytes = &buf[varint_bytes_read..(varint_bytes_read + length as usize)];

        match str::from_utf8(string_bytes) {
            Ok(s) => Ok((s.to_string(), total_bytes_read)),
            Err(e) => Err(CompactStringParseError::InvalidUtf8(e)),
        }
    }

    /// Creates a new `CompactString` from the given byte buffer.
    ///
    /// This function decodes the buffer, extracting a compact string (with a length prefix) and returns a new `CompactString` instance.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice containing the encoded compact string. The first part of the buffer contains a varint encoding the length of the string, followed by the string itself in UTF-8 format.
    ///
    /// # Returns
    ///
    /// Returns a `Result<CompactString, CompactStringParseError>`:
    /// - `Ok(CompactString)` containing the decoded string and its size.
    /// - `Err(CompactStringParseError)` if the buffer is malformed, contains an invalid length prefix, or has an invalid UTF-8 string.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The length encoded in the buffer is invalid or exceeds the remaining buffer size.
    /// - The UTF-8 decoding of the string fails.
    ///
    pub fn new(buf: &[u8]) -> Result<CompactString, CompactStringParseError> {
        let (value, size_len_bytes) = Self::get(buf)?;
        Ok(CompactString {
            size: value.len(),
            value,
            size_len_bytes,
        })
    }
}

impl Decode<CompactString> for [u8] {
    fn decode(buf: &[u8]) -> Result<CompactString, crate::rpc::decode::DecodeError> {
        match CompactString::new(buf) {
            Ok(val) => Ok(val),
            Err(e) => Err(DecodeError::InvalidBuffer(format!(
                "Could not parse compact string from buffer: {e:?}",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_very_long_str() -> String {
        "0123456789".repeat(100).to_string()
    }

    fn generate_test_data() -> Vec<u8> {
        // The length of the string (1000 characters)
        let length = 1000u64;

        // Encode the length as varint (this is a simple implementation for the sake of example)
        let mut varint_bytes = vec![];
        let mut len = length;
        while len > 0x7f {
            varint_bytes.push((len as u8 & 0x7f) | 0x80);
            len >>= 7;
        }
        varint_bytes.push(len as u8);

        // The string itself, just repeat "0" to "9" to make a 1000 character string
        let string_content = gen_very_long_str();

        // Convert string to bytes
        let string_bytes = string_content.as_bytes();

        // Combine the varint length and the string bytes
        let mut buf = varint_bytes;
        buf.extend_from_slice(string_bytes);

        buf
    }

    #[test]
    fn test_parse_string_valid_short() {
        let data: &[u8] = &[5, 104, 101, 108, 108, 111];
        assert_eq!(CompactString::get(data).unwrap().0, "hello".to_string());
    }

    #[test]
    fn test_parse_string_valid_long_varint() {
        let test_data = generate_test_data();
        assert_eq!(
            CompactString::get(&test_data[..]).unwrap().0,
            gen_very_long_str()
        );
    }

    #[test]
    fn test_parse_string_invalid_utf8() {
        let invalid_utf8: &[u8] = &[1, 0xFF];
        let compact = CompactString::get(invalid_utf8);
        assert!(compact.is_err());
    }

    #[test]
    fn test_parse_string_invalid_length() {
        let invalid_length: &[u8] = &[5, 104, 101];
        let compact = CompactString::get(invalid_length);
        assert!(compact.is_err());
    }

    #[test]
    fn test_new_valid_input() {
        let data = generate_test_data();

        let result = CompactString::new(&data);
        assert!(result.is_ok());

        let compact_string = result.unwrap();
        assert_eq!(compact_string.value, gen_very_long_str());
        assert_eq!(compact_string.size, 1000);
        assert_eq!(compact_string.size_len_bytes, 1002);
    }

    // Test invalid length prefix (length is greater than available buffer)
    #[test]
    fn test_new_invalid_length_prefix() {
        // Length 10 in varint (but we only have 6 bytes in the buffer)
        let data: &[u8] = &[10, 104, 101, 108, 108, 111];

        let result = CompactString::new(data);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CompactStringParseError::InvalidLengthPrefix
        );
    }

    // Test invalid UTF-8 encoding
    #[test]
    fn test_new_invalid_utf8() {
        // Non-UTF-8 byte sequence
        let data: &[u8] = &[5, 0, 255, 0, 255, 0]; // Invalid UTF-8 sequence

        let result = CompactString::new(data);
        assert!(result.is_err());
    }

    // Test empty buffer (should fail due to lack of data to decode)
    #[test]
    fn test_new_empty_buffer() {
        let data: &[u8] = &[];

        let result = CompactString::new(data);
        assert!(result.is_err());
        // Depending on the implementation, this might be an InvalidLengthPrefix error or another error type.
        // Adjust the expected error accordingly.
    }

    // Test buffer with length larger than available data (edge case)
    #[test]
    fn test_new_large_length() {
        let data: &[u8] = &[100, 104, 101, 108, 108, 111];

        let result = CompactString::new(data);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            CompactStringParseError::InvalidLengthPrefix
        );
    }
}
