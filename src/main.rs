mod kafka;
mod iam;

use crate::kafka::IamClientContext;
use aws_types::region::Region;
use clap::{Args, Parser, Subcommand, ValueEnum};
use rdkafka::admin::AdminOptions;
use rdkafka::ClientConfig;
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

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct ClusterArgs {
    #[command(subcommand)]
    command: Option<ClusterCommands>,
}

#[derive(Debug, Subcommand)]
enum ClusterCommands {
    Brokers,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct TopicsArgs {
    #[command(subcommand)]
    command: Option<TopicsCommands>,
}

#[derive(Debug, Subcommand)]
enum TopicsCommands {
    List,
    Delete(TopicsDeleteArgs),
}

#[derive(Debug, Args)]
struct TopicsListArgs {}

#[derive(Debug, Args)]
struct TopicsDeleteArgs {
    #[arg(short, long)]
    topic_name: Option<String>,
    #[arg(short, long)]
    run: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client_config = kafka::create_config(cli.bootstrap_servers, cli.iam_auth);
    let aws_region = String::from(cli.aws_region.to_owned());
    let region = Region::new(aws_region);
    let context =
        IamClientContext::new(region, Handle::current());

    match cli.command {
        Commands::Cluster(cluster) => {
            let cluster_cmd = cluster.command.unwrap_or(ClusterCommands::Brokers);
            match cluster_cmd {
                ClusterCommands::Brokers => {
                    list_brokers_cmd(client_config, context, cli.timeout)
                }
            }
        }
        Commands::Topics(topics) => {
            let topics_cmd = topics.command.unwrap_or(TopicsCommands::List);
            match topics_cmd {
                TopicsCommands::List => {
                    list_topics_cmd(client_config, context, cli.timeout);
                }
                TopicsCommands::Delete(args) => {
                    delete_topics_cmd(client_config, context, args.run, args.topic_name, cli.timeout).await;
                }
            }
        }
    }
}

fn list_brokers_cmd(client_config: ClientConfig, context: IamClientContext, timeout: u64) {
    let brokers = kafka::list_brokers(client_config, context, timeout);
    brokers.iter().for_each(|broker| {
        println!("[{}] {}:{}", broker.id(), broker.host(), broker.port())
    });
}

fn list_topics_cmd(config: ClientConfig, context: IamClientContext, timeout: u64) {
    println!("Listing topics");

    let topics = kafka::list_topics(config, context, timeout);
    topics.iter().for_each(|topic| println!("{topic}"));
}

async fn delete_topics_cmd(client_config: ClientConfig, context: IamClientContext, run: bool, topic_name: Option<String>, timeout: u64) {
    let topics = kafka::list_topics(client_config.clone(), context.clone(), timeout);
    let delete_topics: Vec<&str> = topics
        .iter()
        .filter(|topic|
            match topic_name.to_owned() {
                Some(topic_name) => topic.starts_with(topic_name.as_str()),
                None => true
            }
        )
        .map(|topic| topic.as_str())
        .collect();

    if run {
        println!("Deleting topics: {delete_topics:?}");
        let admin_options = AdminOptions::new();
        let result = kafka::create_admin_client(client_config, context.clone()).delete_topics(&delete_topics, &admin_options).await;
        match result {
            Ok(topic_results) => {
                topic_results.iter()
                    .filter_map(|res| {
                        if res.is_err() {
                            Some(res.clone().unwrap_err())
                        } else {
                            None
                        }
                    })
                    .for_each(|(topic, error)| {
                        println!("Unable to delete topic {topic}: {error}");
                    })
            }
            Err(err) => {
                println!("Failed to delete topics: {err}");
            }
        }
    } else {
        println!("Dry run: {delete_topics:?}");
    }
}