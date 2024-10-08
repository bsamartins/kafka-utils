use crate::kafka::client::{create_base_client, IamClientContext};
use crate::kafka::types::ListBrokerEntry;
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;
use std::time::Duration;

pub fn list_brokers<'a>(config: ClientConfig, context: IamClientContext, timeout: Duration) -> Vec<ListBrokerEntry> {
    let result = create_base_client(config, context)
        .fetch_metadata(None, timeout);

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

