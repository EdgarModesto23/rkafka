use bytes::BytesMut;

use crate::rpc::decode::DecodeError;

pub mod requests;

pub trait Respond {
    fn get_response(&self) -> Result<BytesMut, DecodeError>;
}
