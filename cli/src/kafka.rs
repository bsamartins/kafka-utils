use crate::iam::generate_auth_token;
use crate::types::{Broker, ListedTopic};
use aws_types::region::Region;
use itertools::Itertools;
use rdkafka::admin::AdminClient;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext};
use rdkafka::metadata::MetadataTopic;
use rdkafka::{ClientConfig, ClientContext};
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::time::timeout;

pub fn create_config(bootstrap_servers: String, iam_auth: bool) -> ClientConfig {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", bootstrap_servers);
    if iam_auth {
        println!("Using iam authentication");
        config.set("security.protocol", "sasl_ssl");
        config.set("sasl.mechanisms", "OAUTHBEARER");
    }
    config
}

pub fn create_base_client(config: ClientConfig, context: IamClientContext) -> BaseConsumer<IamClientContext> {
    config
        .create_with_context(context)
        .expect("Consumer creation failed")
}

pub fn create_admin_client(config: ClientConfig, context: IamClientContext) -> AdminClient<IamClientContext> {
    config
        .create_with_context(context)
        .expect("admin client creation failed")
}

pub fn list_topics(config: ClientConfig, context: IamClientContext, timeout: Duration) -> Vec<ListedTopic> {
    let client = create_base_client(config, context);
    let metadata = client
        .fetch_metadata(None, timeout)
        .expect("Failed to fetch metadata");

    let topics_metadata = metadata.topics();
    let topic_offsets = fetch_topics_offsets(client, timeout, topics_metadata);

    let mut topics = topics_metadata
        .iter().map(|topic| {
            let message_count = match topic_offsets.get(topic.name()) {
                Some(topic_offset) => {
                    topic_offset.iter().map(|(_, (min, max))| max - min)
                        .sum()
                },
                None => 0,
            };

            ListedTopic {
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
    )
        .into_group_map()
}

pub fn list_topics_names(config: ClientConfig, context: IamClientContext, timeout: Duration) -> Vec<String> {
    let result = create_base_client(config, context)
        .fetch_metadata(None, timeout);

    let mut topics = result.expect("Failed to fetch metadata").topics()
        .iter().map(|topic| topic.name().to_string())
        .collect::<Vec<_>>();
    topics.sort_by_key(|t| t.to_string());
    topics
}

pub fn list_brokers<'a>(config: ClientConfig, context: IamClientContext, timeout: Duration) -> Vec<Broker> {
    let result = create_base_client(config, context)
        .fetch_metadata(None, timeout);

    result.expect("Failed to fetch metadata")
        .brokers()
        .iter()
        .map(|broker|
            Broker {
                id: broker.id(),
                host: broker.host().to_string(),
                port: broker.port(),
            }
        )
        .collect::<Vec<_>>()
}

#[derive(Clone)]
pub struct IamClientContext {
    region: Region,
    rt: Handle,
}

impl IamClientContext {
    pub fn new(region: Region, rt: Handle) -> Self {
        Self { region, rt }
    }
}
impl ClientContext for IamClientContext {
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;
    fn generate_oauth_token(&self, _oauthbearer_config: Option<&str>) -> Result<OAuthToken, Box<dyn Error>> {
        let region = self.region.clone();
        let rt = self.rt.clone();
        let (token, expiration_time_ms) = {
            let handle = thread::spawn(move || {
                rt.block_on(async {
                    timeout(Duration::from_secs(10), generate_auth_token(region.clone())).await
                })
            });
            handle.join().unwrap()??
        };
        Ok(OAuthToken {
            token,
            principal_name: "".to_string(),
            lifetime_ms: expiration_time_ms,
        })
    }
}

impl ConsumerContext for IamClientContext {}