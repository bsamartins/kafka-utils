use std::error::Error;
use std::thread;
use std::time::Duration;
use aws_types::region::Region;
use rdkafka::admin::AdminClient;
use rdkafka::{ClientConfig, ClientContext};
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, ConsumerContext};
use tokio::runtime::Handle;
use tokio::time::timeout;
use crate::kafka::iam::generate_auth_token;

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
