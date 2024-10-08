use crate::kafka::IamClientContext;
use clap::{Args, Subcommand};
use rdkafka::ClientConfig;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ConsumerArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ConsumerCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConsumerCommands {
    List,
}

pub(crate) fn list(p0: ClientConfig, p1: IamClientContext, p2: u64) {
    todo!()
}