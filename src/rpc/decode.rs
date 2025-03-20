use std::fmt;

use thiserror::Error;

#[derive(Error)]
pub enum DecodeError {
    InvalidBuffer(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBuffer(t) => {
                write!(f, "Error while decoding buffer: {t}")
            }
        }
    }
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBuffer(t) => {
                write!(f, "Error while decoding buffer: {t}")
            }
        }
    }
}

pub trait Decode<T> {
    /// A trait for decoding a type `T` from a byte buffer.
    ///
    /// This trait defines a method for decoding a type `T` from a byte slice (`&[u8]`).
    /// The method should implement the logic to parse the byte buffer and return a result containing
    /// either the decoded type `T` or an error.
    ///
    /// # Associated Type
    /// - `T`: The type that will be decoded from the byte buffer.
    ///
    /// # Methods
    ///
    /// - `decode(buf: &[u8]) -> Result<T, Error>`:
    ///   - **buf**: A reference to the byte slice that contains the encoded data.
    ///   - Returns a `Result<T, Error>`, where `T` is the decoded value and `Error` is an error type that occurs during    decoding.
    ///
    /// # Errors
    ///
    /// This method may fail to parse a value if the buffer passed is not able to fit the type:
    /// ex: decode<i16>([0x00])
    fn decode(buf: &[u8]) -> Result<T, DecodeError>;
}

impl Decode<i32> for [u8] {
    fn decode(buf: &[u8]) -> Result<i32, DecodeError> {
        if buf.len() != 4 {
            return Err(DecodeError::InvalidBuffer(
                "Buffer must be exactly 4 bytes for i32".to_string(),
            ));
        }

        match buf.try_into() {
            Ok(bytes) => Ok(i32::from_be_bytes(bytes)),
            Err(e) => Err(DecodeError::InvalidBuffer(format!(
                "Failed to convert buffer to byte array: {e}"
            ))),
        }
    }
}

impl Decode<u64> for [u8] {
    fn decode(buf: &[u8]) -> Result<u64, DecodeError> {
        if buf.len() != 8 {
            return Err(DecodeError::InvalidBuffer(
                "Buffer must be exactly 4 bytes for i32".to_string(),
            ));
        }

        match buf.try_into() {
            Ok(bytes) => Ok(u64::from_be_bytes(bytes)),
            Err(e) => Err(DecodeError::InvalidBuffer(format!(
                "Failed to convert buffer to byte array: {e}"
            ))),
        }
    }
}
