use crate::cmd::table;
use clap::{Args, Subcommand};
use common::kafka;
use common::kafka::client::IamClientContext;
use common::kafka::types::ListBrokerEntry;
use rdkafka::ClientConfig;
use std::borrow::Cow;
use std::time::Duration;
use tabled::Tabled;

pub fn list_brokers_cmd(client_config: ClientConfig, context: IamClientContext, timeout: Duration) {
    let brokers = kafka::broker::list_brokers(client_config, context, timeout)
        .iter().map(|e| ListBrokerTable(e.clone()))
        .collect();
    println!("{}", table::create(brokers))
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ClusterArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ClusterCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ClusterCommands {
    Brokers,
}

struct ListBrokerTable(ListBrokerEntry);

impl Tabled for ListBrokerTable {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.id.to_string().into(),
            self.0.host.as_str().into(),
            self.0.port.to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "ID".into(),
            "Host".into(),
            "Port".into(),
        ]
    }
}
