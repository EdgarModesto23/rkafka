use anyhow::Error;
use bytes::BytesMut;
use types::nullstring::{NullableString, NullableStringError};

use crate::rpc::encode::Encode;

pub mod schema;
pub mod types;

pub struct ResponseBase {
    pub size: i32,
    pub correlation_id: i32,
}

impl ResponseBase {
    #[must_use]
    pub fn new(size: i32, correlation_id: i32) -> ResponseBase {
        ResponseBase {
            size,
            correlation_id,
        }
    }
}

impl Encode for ResponseBase {
    fn encode(&self, buf: &mut BytesMut) {
        self.size.encode(buf);
        self.correlation_id.encode(buf);
    }
}

pub struct RequestBase {
    pub size: i32,
    pub api_key: i16,
    pub api_version: i16,
    pub correlation_id: i32,
    pub client_id: NullableString,
    pub base_size: i16,
}

impl RequestBase {
    /// Creates a new `RequestBase` from the provided byte buffer (`buf`).
    ///
    /// This function parses various fields from the byte buffer, extracting values using Big Endian byte order.
    /// It expects the buffer to contain the following data at specific indices:
    ///
    /// - The first 4 bytes represent the `size` (i32).
    /// - The next 2 bytes represent the `api_key` (i16).
    /// - The following 2 bytes represent the `api_version` (i16).
    /// - The next 8 bytes represent the `correlation_id` (i32).
    /// - A string value, represented by a length field (2 bytes at index 16) and a UTF-8 string, which is parsed into `client_id` (using the `NullableString::new` function).
    /// - The `base_size` is calculated from the sum of a fixed size (15) and the length (from index 16) in the buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A reference to a `BytesMut` buffer containing the data to parse.
    ///
    /// # Returns
    ///
    /// Returns a `Result<RequestBase, Error>`. On success, it returns a `RequestBase` object containing the parsed fields.
    /// If an error occurs during parsing (such as invalid length data or byte conversion failure), it returns an error.
    ///
    /// # Errors
    ///
    /// The function may return an error in the following cases:
    /// - `NullableStringError::Other`: If the length data cannot be converted from the byte slice.
    /// - Other byte conversion errors while parsing the fields.
    pub fn new(buf: &BytesMut) -> Result<RequestBase, Error> {
        if buf.len() < 14 {
            return Err(NullableStringError::InvalidBufLength.into());
        }
        let client_id_size = i16::from_be_bytes(buf[12..14].try_into().map_err(|_| {
            NullableStringError::Other(
                "Failed to convert length from bytes at index 16".to_string(),
            )
        })?);

        let (base_size, client_id) = match client_id_size.cmp(&-1) {
            std::cmp::Ordering::Equal => (14, NullableString::new_empty()),
            _ => (
                14 + client_id_size,
                NullableString::new(buf, 14, client_id_size)?,
            ),
        };
        Ok(RequestBase {
            size: i32::from_be_bytes(buf[0..4].try_into()?),
            api_key: i16::from_be_bytes(buf[4..6].try_into()?),
            api_version: i16::from_be_bytes(buf[6..8].try_into()?),
            correlation_id: i32::from_be_bytes(buf[8..12].try_into()?),
            client_id,
            base_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    // Test case 1: Normal valid buffer with all fields correctly parsed.
    #[test]
    fn test_valid_request_base() {
        let buf = BytesMut::from(
            &[
                0, 0, 0, 10, // size (i32)
                0, 1, // api_key (i16)
                0, 1, // api_version (i16)
                0, 0, 0, 5, // correlation_id (i32)
                0, 5, // client_id_size (i16)
                72, 101, 108, 108, 111, // client_id ("Hello" in UTF-8 bytes)
            ][..],
        );

        let result = RequestBase::new(&buf);
        assert!(result.is_ok());

        let request_base = result.unwrap();
        assert_eq!(request_base.size, 10);
        assert_eq!(request_base.api_key, 1);
        assert_eq!(request_base.api_version, 1);
        assert_eq!(request_base.correlation_id, 5);
        assert_eq!(request_base.base_size, 19);
        assert_eq!(request_base.client_id.value, "Hello");
        assert_eq!(request_base.client_id.length, 5);
    }

    // Test case 2: Buffer is too small to parse the required fields (less than 14 bytes).
    #[test]
    fn test_buffer_too_small() {
        let buf = BytesMut::from(&[0u8, 0u8, 0u8, 10u8, 0u8, 1u8][..]); // Only 6 bytes
        let result = RequestBase::new(&buf);
        assert!(result.is_err());
    }

    // Test case 3: Buffer with invalid client_id size that cannot be parsed.
    #[test]
    fn test_invalid_client_id_size() {
        let buf = BytesMut::from(
            &[
                0, 0, 0, 10, // size (i32)
                0, 1, // api_key (i16)
                0, 1, // api_version (i16)
                0, 0, 0, 5, // correlation_id (i32)
                0, 3, // client_id_size (i16)
                72, 101, 0xFF, // Invalid UTF-8 byte in client_id
            ][..],
        );

        let result = RequestBase::new(&buf);
        assert!(result.is_err());
    }

    // Test case 4: Buffer with zero-length client_id.
    #[test]
    fn test_zero_length_client_id() {
        let buf = BytesMut::from(
            &[
                0, 0, 0, 10, // size (i32)
                0, 1, // api_key (i16)
                0, 1, // api_version (i16)
                0, 0, 0, 5, // correlation_id (i32)
                255, 255, // client_id_size (i16)
            ][..],
        );

        let result = RequestBase::new(&buf);
        assert!(result.is_ok());

        let request_base = result.unwrap();
        assert_eq!(request_base.size, 10);
        assert_eq!(request_base.api_key, 1);
        assert_eq!(request_base.api_version, 1);
        assert_eq!(request_base.correlation_id, 5);
        assert_eq!(request_base.base_size, 14);
        assert_eq!(request_base.client_id.value, "");
        assert_eq!(request_base.client_id.length, 0);
    }

    // Test case 5: Buffer where the client_id size is larger than the buffer can handle.
    #[test]
    fn test_client_id_size_exceeds_buffer() {
        let buf = BytesMut::from(
            &[
                0, 0, 0, 10, // size (i32)
                0, 1, // api_key (i16)
                0, 1, // api_version (i16)
                0, 0, 0, 5, // correlation_id (i32)
                0, 100, // client_id_size (i16) — too large for the remaining buffer
                0, 0,
            ][..],
        );

        let result = RequestBase::new(&buf);
        assert!(result.is_err());
    }

    // Test case 6: Buffer with all fields correctly set, including a small client_id.
    #[test]
    fn test_small_client_id() {
        let buf = BytesMut::from(
            &[
                0, 0, 0, 10, // size (i32)
                0, 1, // api_key (i16)
                0, 1, // api_version (i16)
                0, 0, 0, 5, // correlation_id (i32)
                0, 1,  // client_id_size (i16) — single character
                72, // client_id ("H")
            ][..],
        );

        let result = RequestBase::new(&buf);
        assert!(result.is_ok());

        let request_base = result.unwrap();
        assert_eq!(request_base.size, 10);
        assert_eq!(request_base.api_key, 1);
        assert_eq!(request_base.api_version, 1);
        assert_eq!(request_base.correlation_id, 5);
        assert_eq!(request_base.base_size, 15);
        assert_eq!(request_base.client_id.value, "H");
        assert_eq!(request_base.client_id.length, 1);
    }
}
