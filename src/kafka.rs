use rdkafka::admin::AdminClient;
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::ClientConfig;
use std::time::Duration;

pub fn create_config(bootstrap_servers: String, iam_auth: bool) -> ClientConfig {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", bootstrap_servers);
    config
}

pub fn create_base_client(config: ClientConfig) -> BaseConsumer {
    config
        .create()
        .expect("Consumer creation failed")
}

pub fn create_admin_client(config: ClientConfig) -> AdminClient<DefaultClientContext> {
    config
        .create()
        .expect("admin client creation failed")
}

pub fn list_topics(config: ClientConfig) -> Vec<String> {
    let result = create_base_client(config)
        .fetch_metadata(None, Duration::from_secs(30));

    let mut topics = result.expect("Failed to fetch metadata").topics()
        .iter().map(|topic| topic.name().to_string())
        .collect::<Vec<_>>();
    topics.sort();
    topics
}
