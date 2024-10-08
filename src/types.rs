#[derive(Debug)]
pub struct ListedTopic {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub message_count: i64,
    pub size: i64,
}

#[derive(Debug)]
pub struct ListedConsumerGroup {
    pub name: String,
}

pub struct Broker {
    pub id: i32,
    pub host: String,
    pub port: i32,
}