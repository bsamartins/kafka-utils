#[derive(Debug, Clone)]
pub struct ListTopicEntry {
    pub name: String,
    pub partitions: i32,
    pub replication_factor: i32,
    pub message_count: i64,
    pub size: i64,
}

#[derive(Debug, Clone)]
pub struct ListConsumerGroupEntry {
    pub name: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct ListBrokerEntry {
    pub id: i32,
    pub host: String,
    pub port: i32,
}