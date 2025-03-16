use crate::protocol::{
    types::{compactarray::CompactArray, compactstring::CompactString},
    RequestBase,
};

pub struct DescribeTopicPartitions {
    pub base_request: RequestBase,
    pub topics_array: CompactArray<CompactString>,
    pub response_partition_limit: i32,
    pub cursor: i8,
    pub tag_buffer: i8,
}
