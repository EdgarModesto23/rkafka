use bytes::BytesMut;

use crate::rpc::decode::DecodeError;

pub mod requests;

pub mod responses;

pub trait Respond {
    fn get_response(&self) -> Result<BytesMut, DecodeError>;
}
