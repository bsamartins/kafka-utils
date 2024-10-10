use crate::kafka::client::{create_base_client, Config};
use crate::kafka::types::ListBrokerEntry;
use rdkafka::consumer::Consumer;

pub fn list_brokers<'a>(config: &Config) -> Vec<ListBrokerEntry> {
    let result = create_base_client(config)
        .fetch_metadata(None, config.timeout);

    result.expect("Failed to fetch metadata")
        .brokers()
        .iter()
        .map(|broker|
            ListBrokerEntry {
                id: broker.id(),
                host: broker.host().to_string(),
                port: broker.port(),
            }
        )
        .collect::<Vec<_>>()
}

