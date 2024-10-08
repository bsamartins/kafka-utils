use crate::kafka::client::{create_base_client, Config, IamClientContext};
use crate::kafka::types::ListTopicEntry;
use itertools::Itertools;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::metadata::MetadataTopic;
use std::collections::HashMap;
use std::time::Duration;

pub fn list_topics(config: Config) -> Vec<ListTopicEntry> {
    let client = create_base_client(config.clone());
    let metadata = client
        .fetch_metadata(None, config.timeout)
        .expect("Failed to fetch metadata");

    let topics_metadata = metadata.topics();
    let topic_offsets = fetch_topics_offsets(client, config.timeout, topics_metadata);

    let mut topics = topics_metadata
        .iter().map(|topic| {
        let message_count = match topic_offsets.get(topic.name()) {
            Some(topic_offset) => {
                topic_offset.iter().map(|(_, (min, max))| max - min)
                    .sum()
            },
            None => 0,
        };

        ListTopicEntry {
            name: topic.name().to_string(),
            partitions: topic.partitions().iter().len() as i32,
            replication_factor: topic.partitions().iter()
                .flat_map(|partition| partition.replicas().iter())
                .max()
                .map(|v| *v)
                .unwrap_or_else(|| 0),
            message_count,
            size: 0,
        }
    })
        .collect::<Vec<_>>();
    topics.sort_by(|a, b| a.name.cmp(&b.name));
    topics
}

fn fetch_topics_offsets(client: BaseConsumer<IamClientContext>, timeout: Duration, topics_metadata: &[MetadataTopic]) -> HashMap<&str, Vec<(i32, (i64, i64))>> {
    topics_metadata.iter()
        .flat_map(|topic|
            topic.partitions()
                .iter()
                .map(|partition| (topic.name(), partition.id()))
        )
        .map(|(topic, partition_id)|
            (topic, (partition_id, client.fetch_watermarks(topic, partition_id, timeout)))
        ).filter_map(|(topic, (partition_id, offset_result))|
        match offset_result {
            Ok(offset) => Some((topic, (partition_id, offset))),
            Err(_) => None
        }
    ).into_group_map()
}

pub fn list_topics_names(config: Config) -> Vec<String> {
    let result = create_base_client(config.clone())
        .fetch_metadata(None, config.timeout);

    let mut topics = result.expect("Failed to fetch metadata").topics()
        .iter().map(|topic| topic.name().to_string())
        .collect::<Vec<_>>();
    topics.sort_by_key(|t| t.to_string());
    topics
}
