#[derive(Debug)]
pub struct ListedTopic {
    pub name: String,
}

pub struct Broker {
    pub id: i32,
    pub host: String,
    pub port: i32,
}