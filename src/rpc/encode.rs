use bytes::BytesMut;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

impl Encode for i32 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(&i32::to_be_bytes(*self)[..]);
    }
}
