use bytes::BytesMut;

pub trait Encode<T> {
    fn encode(&self, buf: &BytesMut);
}
