use super::{compactarray::CompactArray, Offset};

pub struct Partition {
    pub size: u64,
    pub node_id: i32,
    pub leader: i32,
    pub leader_epoch: i32,
    pub replica_nodes: CompactArray<i32>,
    pub in_sync_nodes: CompactArray<i32>,
    pub eligible_leader_replicas: CompactArray<i32>,
    pub last_known_elr: CompactArray<i32>,
    pub offline_replicas: CompactArray<i32>,
    pub tag_buffer: u8,
}

impl Offset for Partition {
    fn get_offset(&self) -> u64 {
        self.size
    }
}

impl Partition {
    pub fn new(
        size: u64,
        node_id: i32,
        leader: i32,
        leader_epoch: i32,
        replica_nodes: CompactArray<i32>,
        in_sync_nodes: CompactArray<i32>,
        eligible_leader_replicas: CompactArray<i32>,
        last_known_elr: CompactArray<i32>,
        offline_replicas: CompactArray<i32>,
        tag_buffer: u8,
    ) -> Partition {
        Partition {
            size,
            node_id,
            leader,
            leader_epoch,
            replica_nodes,
            in_sync_nodes,
            eligible_leader_replicas,
            last_known_elr,
            offline_replicas,
            tag_buffer,
        }
    }
}
