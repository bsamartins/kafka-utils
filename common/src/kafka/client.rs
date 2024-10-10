use crate::kafka::iam::generate_auth_token;
use aws_types::region::Region;
use rdkafka::admin::AdminClient;
use rdkafka::client::OAuthToken;
use rdkafka::consumer::{BaseConsumer, ConsumerContext};
use rdkafka::{ClientConfig, ClientContext};
use std::error::Error;
use std::thread;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::time::timeout;

pub fn create_config(bootstrap_servers: String, iam_auth: bool, region: String, timeout: Duration) -> Config {
    let aws_region = Region::new(region);
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", bootstrap_servers);
    if iam_auth {
        println!("Using iam authentication");
        config.set("security.protocol", "sasl_ssl");
        config.set("sasl.mechanisms", "OAUTHBEARER");
    }
    Config {
        client_config: config,
        context: IamClientContext::new(aws_region, Handle::current()),
        timeout,
    }
}

pub fn create_base_client(config: &Config) -> BaseConsumer<IamClientContext> {
    config
        .client_config
        .create_with_context(config.context.clone())
        .expect("Consumer creation failed")
}

pub fn create_admin_client(config: &Config) -> AdminClient<IamClientContext> {
    config
        .client_config
        .create_with_context(config.context.to_owned())
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

#[derive(Clone)]
pub struct Config {
    client_config: ClientConfig,
    context: IamClientContext,
    pub(crate) timeout: Duration,
}