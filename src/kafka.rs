use crate::iam::generate_auth_token;
use aws_types::region::Region;
use rdkafka::admin::AdminClient;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext};
use rdkafka::{ClientConfig, ClientContext};
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

pub fn list_topics(config: ClientConfig, context: IamClientContext, timeout: u64) -> Vec<String> {
    let result = create_base_client(config, context)
        .fetch_metadata(None, Duration::from_millis(timeout));

    let mut topics = result.expect("Failed to fetch metadata").topics()
        .iter().map(|topic| topic.name().to_string())
        .collect::<Vec<_>>();
    topics.sort();
    topics
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