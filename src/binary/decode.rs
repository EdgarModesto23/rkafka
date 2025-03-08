use anyhow::Error;

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
    fn decode(buf: &[u8]) -> Result<T, Error>;
}
