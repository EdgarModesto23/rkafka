use std::fmt::Debug;

use bytes::{BufMut, BytesMut};

use crate::{
    protocol::{
        schema::Respond,
        types::{
            compactarray::CompactArray, compactstring::CompactString, topicstr::TopicStr,
            CompactEncode,
        },
        RequestBase,
    },
    rpc::encode::Encode,
};

pub struct DescribeTopicPartitions {
    pub base_request: RequestBase,
    pub topics_array: CompactArray<TopicStr>,
    pub response_partition_limit: i32,
    pub cursor: u8,
    pub tag_buffer: u8,
}

pub struct Topic<'a> {
    error: u16,
    name: &'a CompactString,
    id: [u8; 16],
    is_internal: u8,
    partitions: CompactArray<CompactString>,
    authorized_operations: u32,
    tag_buffer: u8,
}

impl Encode for Topic<'_> {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.error);
        self.name.encode_compact(buf);
        buf.put(&self.id[..]);
        buf.put_u8(self.is_internal);
        self.partitions.encode(buf);
        buf.put(&self.authorized_operations.to_be_bytes()[..]);
        buf.put_u8(self.tag_buffer);
    }
}

impl Topic<'_> {
    fn new(name: &CompactString) -> Result<Topic, anyhow::Error> {
        println!("{name:?}");
        Ok(Topic {
            error: 3,
            name,
            id: [0x00; 16],
            is_internal: 0,
            partitions: CompactArray { elements: vec![] },
            authorized_operations: 0x0000_0df8,
            tag_buffer: 0,
        })
    }
}

impl Debug for Topic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Topic")
            .field("name", &self.name.value)
            .finish()
    }
}

impl DescribeTopicPartitions {
    pub fn new(
        base_request: RequestBase,
        buf: &[u8],
    ) -> Result<DescribeTopicPartitions, anyhow::Error> {
        let (topics_array, offset) = CompactArray::<TopicStr>::new(buf)?;
        let response_partition_limit = i32::from_be_bytes(buf[offset..(offset + 4)].try_into()?);
        Ok(DescribeTopicPartitions {
            base_request,
            topics_array,
            response_partition_limit,
            cursor: 0xff,
            tag_buffer: 0x00,
        })
    }
}

impl Respond for DescribeTopicPartitions {
    fn get_response(&self) -> Result<bytes::BytesMut, crate::rpc::decode::DecodeError> {
        let mut message = BytesMut::new();
        message.put_i32(self.base_request.correlation_id);
        message.put_u8(0x00);
        message.put(&[0x00, 0x00, 0x00, 0x00][..]);
        message.put(&((self.topics_array.elements.len() + 1) as u8).to_be_bytes()[..]);
        let _ = self.topics_array.elements.iter().try_for_each(
            |topic: &TopicStr| -> Result<(), anyhow::Error> {
                let topic = Topic::new(&topic.value)?;
                topic.encode(&mut message);
                Ok(())
            },
        );
        message.put_u8(self.cursor);
        message.put_u8(self.tag_buffer);
        let mut response = BytesMut::with_capacity(message.len() + 4);
        let len = (message.len()) as i32;
        response.put(&len.to_be_bytes()[..]);
        response.put(&message[..]);
        response.resize(response.capacity(), 0);

        Ok(response)
    }
}
