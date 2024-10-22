mod cmd;

use crate::cmd::broker::{ClusterArgs, ClusterCommands};
use crate::cmd::consumer::{ConsumerArgs, ConsumerCommands, ListConsumerArgs};
use crate::cmd::topic::{TopicsArgs, TopicsCommands};
use clap::{Parser, Subcommand, ValueEnum};
use common::kafka;
use core::time::Duration;

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

    let config = kafka::client::create_config(
        cli.bootstrap_servers,
        cli.iam_auth,
        cli.aws_region,
        Duration::from_millis(cli.timeout.into()),
    );

    match cli.command {
        Commands::Cluster(cluster) => {
            let cluster_cmd = cluster.command.unwrap_or(ClusterCommands::Brokers);
            match cluster_cmd {
                ClusterCommands::Brokers => {
                    cmd::broker::list_brokers_cmd(&config)
                }
            }
        }
        Commands::Topics(topics) => {
            let topics_cmd = topics.command.unwrap_or(TopicsCommands::List);
            match topics_cmd {
                TopicsCommands::List => {
                    cmd::topic::list_topics_cmd(&config);
                }
                TopicsCommands::Delete(args) => {
                    cmd::topic::delete_topics_cmd(&config, args.run, args.topic_name).await;
                }
            }
        }
        Commands::Consumers(consumer) => {
            let consumer_cmd = consumer.command.unwrap_or(ConsumerCommands::List(ListConsumerArgs { consumer_group: None }));
            match consumer_cmd {
                ConsumerCommands::List(args) => {
                    cmd::consumer::list(&config, args.consumer_group)
                }
                ConsumerCommands::Delete(args) => {
                    cmd::consumer::delete(&config, args.consumer_group).await
                }
            }
        }
    }
}
