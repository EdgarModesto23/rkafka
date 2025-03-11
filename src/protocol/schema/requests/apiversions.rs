use bytes::{BufMut, BytesMut};

use crate::{
    protocol::{
        schema::Respond,
        types::compactstring::{CompactString, CompactStringParseError},
        RequestBase,
    },
    rpc::{decode::DecodeError, encode::Encode},
};

pub struct ApiVersionRequest {
    pub base_request: RequestBase,
    pub client_software_name: CompactString,
    pub client_software_version: CompactString,
}

impl ApiVersionRequest {
    /// Creates a new `ApiVersionRequest` from the provided `RequestBase` and a byte slice.
    ///
    /// This function takes in a `RequestBase` and a byte slice (`buf`), then attempts to
    /// parse the `client_software_name` and `client_software_version` from the provided buffer.
    /// It uses the `CompactString::new` function to parse these components. If parsing is
    /// successful, an `ApiVersionRequest` is returned. If any errors occur during parsing, a
    /// `CompactStringParseError` will be returned.
    ///
    /// # Parameters
    ///
    /// * `base` - The base request metadata (`RequestBase`), typically containing information
    ///   such as the API key, base size, etc.
    /// * `buf` - A byte slice (`&[u8]`) that contains the data used to extract the
    ///   `client_software_name` and `client_software_version`. The buffer is assumed to be
    ///   structured in a specific way expected by the `CompactString::new` function.
    ///
    /// # Returns
    ///
    /// This function returns a `Result`:
    ///
    /// * `Ok(ApiVersionRequest)` - If the parsing succeeds, it returns the created `ApiVersionRequest`.
    /// * `Err(CompactStringParseError)` - If any errors occur during parsing, it returns the error.
    ///
    /// # Errors
    ///
    /// The function may return a `CompactStringParseError` if the parsing of the `client_software_name`
    /// or `client_software_version` fails. This could occur if the buffer is malformed or does not
    /// contain the expected data for either field.
    pub fn new(
        base: RequestBase,
        buf: &[u8],
    ) -> Result<ApiVersionRequest, CompactStringParseError> {
        let client_software_name = CompactString::new(buf)?;
        let client_software_version =
            CompactString::new(&buf[client_software_name.size_len_bytes as usize..])?;
        Ok(ApiVersionRequest {
            base_request: base,
            client_software_name,
            client_software_version,
        })
    }
}

impl Respond for ApiVersionRequest {
    fn get_response(&self) -> Result<bytes::BytesMut, DecodeError> {
        let mut response = BytesMut::new();
        response.put_slice(&[0, 0, 0, 0]);
        self.base_request.correlation_id.encode(&mut response);
        response.put_slice(&[0, 35]);
        Ok(response)
    }
}
