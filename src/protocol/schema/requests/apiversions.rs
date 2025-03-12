use anyhow::Error;
use serde::Deserialize;
use std::{fs::File, io::BufReader, path::Path};

use bytes::{BufMut, BytesMut};

use crate::{
    protocol::{
        schema::Respond,
        types::compactstring::{CompactString, CompactStringParseError},
        RequestBase,
    },
    rpc::{decode::DecodeError, encode::Encode},
};

use super::is_version_supported;

#[derive(Deserialize, Debug)]
pub struct SupportedVersionsKey {
    pub key: i16,
    pub min: i16,
    pub max: i16,
}

fn get_supported_versions_bytes<P: AsRef<Path>>(path: P) -> Result<BytesMut, Error> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut data_bytes = BytesMut::new();

    let data: Vec<SupportedVersionsKey> = serde_json::from_reader(reader)?;

    let arr_size = data.len() as i8 + 1;

    data_bytes.put_i8(arr_size);

    for key in data.iter() {
        println!("{key:?}");
        data_bytes.extend_from_slice(&key.key.to_be_bytes()[..]);
        data_bytes.extend_from_slice(&key.min.to_be_bytes()[..]);
        data_bytes.extend_from_slice(&key.max.to_be_bytes()[..]);
        //tag buffer
        data_bytes.put_u8(0);
    }
    Ok(data_bytes)
}

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
        let data = match get_supported_versions_bytes("supported_versions.json") {
            Ok(supported_keys) => supported_keys,
            Err(e) => {
                return Err(DecodeError::InvalidBuffer(format!(
                    "Error while decoding supported keys: {e:?}"
                )))
            }
        };
        let res_size = (4 + 2 + data.len() + 5) as i32;
        let error: i16 = match is_version_supported(
            "supported_versions.json",
            self.base_request.api_key,
            self.base_request.api_version,
        ) {
            Ok(value) => {
                if value {
                    0
                } else {
                    35
                }
            }
            Err(e) => {
                return Err(DecodeError::InvalidBuffer(format!(
                    "Error while getting supported versions: {e:?}"
                )))
            }
        };
        println!("{res_size:?}");
        res_size.encode(&mut response);
        self.base_request.correlation_id.encode(&mut response);
        response.put_slice(&error.to_be_bytes());
        response.put_slice(&data[..]);
        //throttle ms
        response.put_slice(&[0, 0, 0, 0]);
        //tag buffer
        response.put_u8(0);

        Ok(response)
    }
}
