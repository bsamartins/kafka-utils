mod kafka;
mod iam;
mod types;
mod cmd;

use crate::cmd::broker::{ClusterArgs, ClusterCommands};
use crate::cmd::topic::{TopicsArgs, TopicsCommands};
use crate::cmd::consumer::{ConsumerArgs, ConsumerCommands, ListConsumerArgs};
use crate::kafka::IamClientContext;
use aws_types::region::Region;
use core::time::Duration;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tokio::runtime::Handle;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "kafka-utils")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, required = true)]
    bootstrap_servers: String,
    #[arg(short, long)]
    iam_auth: bool,
    #[arg(short, long, default_value = "10000")]
    timeout: u64,
    #[arg(short, long, default_value = "eu-west-1")]
    aws_region: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Cluster(ClusterArgs),
    #[command(arg_required_else_help = true)]
    Consumers(ConsumerArgs),
    #[command(arg_required_else_help = true)]
    Topics(TopicsArgs),
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl std::fmt::Display for ColorWhen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client_config = kafka::create_config(cli.bootstrap_servers, cli.iam_auth);
    let aws_region = String::from(cli.aws_region.to_owned());
    let region = Region::new(aws_region);
    let context =
        IamClientContext::new(region, Handle::current());
    let timeout = Duration::from_millis(cli.timeout.into());

    match cli.command {
        Commands::Cluster(cluster) => {
            let cluster_cmd = cluster.command.unwrap_or(ClusterCommands::Brokers);
            match cluster_cmd {
                ClusterCommands::Brokers => {
                    cmd::broker::list_brokers_cmd(client_config, context, timeout)
                }
            }
        }
        Commands::Topics(topics) => {
            let topics_cmd = topics.command.unwrap_or(TopicsCommands::List);
            match topics_cmd {
                TopicsCommands::List => {
                    cmd::topic::list_topics_cmd(client_config, context, timeout);
                }
                TopicsCommands::Delete(args) => {
                    cmd::topic::delete_topics_cmd(client_config, context, args.run, args.topic_name, timeout).await;
                }
            }
        }
        Commands::Consumers(consumer) => {
            let consumer_cmd = consumer.command.unwrap_or(ConsumerCommands::List(ListConsumerArgs { consumer_group: None }));
            match consumer_cmd {
                ConsumerCommands::List(args) => {
                    cmd::consumer::list(client_config, context, timeout, args.consumer_group)
                }
                ConsumerCommands::Delete(args) => {
                    cmd::consumer::delete(client_config, context, timeout, args.consumer_group).await
                }
            }
        }
    }
}