use std::time::Duration;
use clap::{Args, Subcommand};
use common::kafka;
use common::kafka::{client::IamClientContext};
use rdkafka::ClientConfig;

pub fn list_brokers_cmd(client_config: ClientConfig, context: IamClientContext, timeout: Duration) {
    let brokers = kafka::broker::list_brokers(client_config, context, timeout);
    brokers.iter().for_each(|broker| {
        println!("[{}] {}:{}", broker.id, broker.host, broker.port)
    });
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
