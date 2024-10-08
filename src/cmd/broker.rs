use clap::{Args, Subcommand};
use crate::kafka;
use crate::kafka::IamClientContext;
use rdkafka::ClientConfig;

pub fn list_brokers_cmd(client_config: ClientConfig, context: IamClientContext, timeout: u64) {
    let brokers = kafka::list_brokers(client_config, context, timeout);
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
