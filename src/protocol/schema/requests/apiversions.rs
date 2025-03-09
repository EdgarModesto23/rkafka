use crate::protocol::{types::compactstring::CompactString, RequestBase};

pub struct ApiVersionRequest {
    pub base_request: RequestBase,
    pub client_software_name: CompactString,
    pub client_software_version: CompactString,
}
