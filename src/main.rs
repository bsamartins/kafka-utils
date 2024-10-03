use clap::{Args, Parser, Subcommand, ValueEnum};
use rdkafka::admin::{AdminClient, AdminOptions};
use rdkafka::client::DefaultClientContext;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::ClientConfig;
use std::time::Duration;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "kafka-utils")]
#[command(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, required = true)]
    bootstrap_servers: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
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

    match cli.command {
        Commands::Topics(topics) => {
            let topics_cmd = topics.command.unwrap_or(TopicsCommands::List);
            match topics_cmd {
                TopicsCommands::List => {
                    list_topics_cmd(cli.bootstrap_servers);
                }
                TopicsCommands::Delete(args) => {
                    delete_topics_cmd(cli.bootstrap_servers, args.run, args.topic_name).await;
                }
            }
        }
    }
}

fn create_config(bootstrap_servers: String) -> ClientConfig {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", bootstrap_servers);
    config
}

fn create_base_client(bootstrap_servers: String) -> BaseConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()
        .expect("Consumer creation failed")
}

fn create_admin_client(bootstrap_servers: String) -> AdminClient<DefaultClientContext> {
    create_config(bootstrap_servers)
        .create()
        .expect("admin client creation failed")
}

fn list_topics_cmd(bootstrap_servers: String) {
    println!("Listing topics");

    let topics = list_topics(bootstrap_servers);
    topics.iter().for_each(|topic| println!("{topic}"));
}

fn list_topics(bootstrap_servers: String) -> Vec<String> {
    let result = create_base_client(bootstrap_servers)
        .fetch_metadata(None, Duration::from_secs(30));

    let mut topics = result.expect("Failed to fetch metadata").topics()
        .iter().map(|topic| topic.name().to_string())
        .collect::<Vec<_>>();
    topics.sort();
    topics
}

async fn delete_topics_cmd(bootstrap_servers: String, run: bool, topic_name: Option<String>) {
    let topics = list_topics(bootstrap_servers.clone());
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
        let result = create_admin_client(bootstrap_servers).delete_topics(&delete_topics, &admin_options).await;
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