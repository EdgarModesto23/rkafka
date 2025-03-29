use super::compactarray::CompactArray;

pub struct TopicRecord {}

pub struct PartitionRecord {
    pub id: i32,
    pub topic_id: String,
    pub replica_array: CompactArray<i32>,
    pub in_sync_replica: CompactArray<i32>,
    pub adding_replicas: CompactArray<i32>,
    pub leader: i32,
    pub leader_epoch: i32,
    pub partition_epoch: i32,
}

pub struct Record<T> {
    pub value: T,
    pub kind: String,
}
