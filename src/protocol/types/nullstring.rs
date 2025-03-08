use core::fmt;

use bytes::BytesMut;

use std::error::Error;

pub struct NullableString {
    pub value: String,
    pub length: i16,
}

pub enum NullableStringError {
    IndexOutOfBounds,
    InvalidBufLength,
    Other(String),
}

impl fmt::Display for NullableStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NullableStringError::IndexOutOfBounds => {
                write!(f, "Index is out of bounds.")
            }
            NullableStringError::Other(value) => {
                write!(f, "{value}")
            }
            NullableStringError::InvalidBufLength => {
                write!(f, "Parsed buf is not a valid int16")
            }
        }
    }
}

impl fmt::Debug for NullableStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NullableStringError::IndexOutOfBounds => {
                write!(f, "Index is out of bounds.")
            }
            NullableStringError::Other(value) => {
                write!(f, "{value}")
            }
            NullableStringError::InvalidBufLength => {
                write!(f, "Parsed buf is not a valid int16")
            }
        }
    }
}

impl Error for NullableStringError {}

impl NullableString {
    /// Creates a new `NullableString` from a byte buffer, starting at the specified index.
    ///
    /// This function extracts a string from the provided byte buffer (`buf`) starting at the given index (`idx`).
    /// The function expects the length of the string to be encoded in the first 2 bytes of the buffer at the given index,
    /// using Big Endian format (i.e., a 16-bit integer representing the length of the string).
    /// It then attempts to read the string from the buffer, convert it to a `String`, and return a `NullableString`.
    ///
    /// # Arguments
    ///
    /// * `buf` - A reference to a `BytesMut` buffer containing the encoded string and its length.
    /// * `idx` - The starting index in the buffer where the length and string are located.
    ///
    /// # Returns
    ///
    /// Returns a `Result<NullableString, NullableStringError>`:
    /// * `Ok(NullableString)` containing the extracted string if the operation is successful.
    /// * `Err(NullableStringError)` if any errors occur, including:
    ///   - `IndexOutOfBounds`: The provided index is out of bounds for the buffer.
    ///   - `InvalidBufLength`: The byte slice at the given index cannot be converted to a valid 16-bit length value.
    ///   - `Other`: A generic error if the string cannot be read from the buffer or converted to a valid UTF-8 string.
    ///
    /// # Errors
    ///
    /// The following errors may be returned:
    /// - `IndexOutOfBounds`: The index `idx` is greater than or equal to the buffer length.
    /// - `InvalidBufLength`: The byte slice starting at `idx` does not contain enough data to extract the length as an `i16`.
    /// - `Other`: A generic error occurs during the conversion of length or reading the UTF-8 string from the buffer. This includes:
    ///   - Failure to convert the byte slice to an `i16` (invalid length encoding).
    ///   - Failure to read a valid UTF-8 string from the byte slice.
    pub fn new(
        buf: &BytesMut,
        idx: usize,
        length: i16,
    ) -> Result<NullableString, NullableStringError> {
        if length == -1 {
            return Ok(NullableString {
                value: String::new(),
                length: 0,
            });
        }

        println!("{idx}: {length}");

        if (idx + (length - 1) as usize) >= buf.len() {
            return Err(NullableStringError::IndexOutOfBounds);
        }

        let range = idx..(idx + length as usize);
        Ok(NullableString {
            value: String::from_utf8(buf[range].into()).map_err(|_| {
                NullableStringError::Other("Failed to read string from bytes".to_string())
            })?,
            length,
        })
    }

    #[must_use]
    pub fn new_empty() -> NullableString {
        NullableString {
            value: String::new(),
            length: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import the NullableString and NullableStringError
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_new_success() {
        let mut buf = BytesMut::with_capacity(10);
        buf.extend_from_slice(&[0, 5]);
        buf.put(&b"Hello"[..]);

        let idx = 2;
        let length = 5;

        let result = NullableString::new(&buf, idx, length);

        let nullable_string = result.unwrap();
        assert_eq!(nullable_string.value, "Hello");
        assert_eq!(nullable_string.length, 5);
    }

    #[test]
    fn test_new_empty_string_more_content() {
        let mut buf = BytesMut::with_capacity(10);
        buf.extend_from_slice(&[0, 255]);
        buf.extend_from_slice(b"Hello");

        let idx = 3;
        let length = -1;

        let result = NullableString::new(&buf, idx, length);

        assert!(result.is_ok());
        let nullable_string = result.unwrap();
        assert_eq!(nullable_string.value, "");
        assert_eq!(nullable_string.length, 0);
    }

    #[test]
    fn test_new_empty_string() {
        let mut buf = BytesMut::with_capacity(10);
        buf.extend_from_slice(&[0, 255]);
        buf.extend_from_slice(b"");

        let idx = 0;
        let length = -1;

        let result = NullableString::new(&buf, idx, length);

        assert!(result.is_ok());
        let nullable_string = result.unwrap();
        assert_eq!(nullable_string.value, "");
        assert_eq!(nullable_string.length, 0);
    }

    #[test]
    fn test_index_out_of_bounds() {
        let mut buf = BytesMut::with_capacity(10);
        buf.extend_from_slice(&[0, 5]);
        buf.extend_from_slice(b"Hello");

        let idx = 0;
        let length = 10;

        let result = NullableString::new(&buf, idx, length);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_buf_length() {
        let mut buf = BytesMut::with_capacity(2); // Only 2 bytes available
        buf.extend_from_slice(&[0, 5]); // Big endian 16-bit length (5 bytes)

        let idx = 0;
        let length = 5; // Requesting 5 bytes but only 2 are available

        let result = NullableString::new(&buf, idx, length);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8() {
        // Invalid UTF-8: Create a buffer with invalid UTF-8 byte sequence
        let mut buf = BytesMut::with_capacity(10);
        buf.extend_from_slice(&[0, 3]); // Big endian 16-bit length (3 bytes)
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Invalid UTF-8 bytes

        let idx = 0;
        let length = 3; // Length of the string

        let result = NullableString::new(&buf, idx, length);

        assert!(result.is_err());
    }
}
