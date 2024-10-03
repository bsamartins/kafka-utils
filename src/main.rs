mod kafka;

use clap::{Args, Parser, Subcommand, ValueEnum};
use rdkafka::admin::AdminOptions;
use rdkafka::ClientConfig;

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
    let client_config = kafka::create_config(cli.bootstrap_servers, cli.iam_auth);

    match cli.command {
        Commands::Topics(topics) => {
            let topics_cmd = topics.command.unwrap_or(TopicsCommands::List);
            match topics_cmd {
                TopicsCommands::List => {
                    list_topics_cmd(client_config);
                }
                TopicsCommands::Delete(args) => {
                    delete_topics_cmd(client_config, args.run, args.topic_name).await;
                }
            }
        }
    }
}

fn list_topics_cmd(config: ClientConfig) {
    println!("Listing topics");

    let topics = kafka::list_topics(config);
    topics.iter().for_each(|topic| println!("{topic}"));
}

async fn delete_topics_cmd(client_config: ClientConfig, run: bool, topic_name: Option<String>) {
    let topics = kafka::list_topics(client_config.clone());
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
        let result = kafka::create_admin_client(client_config).delete_topics(&delete_topics, &admin_options).await;
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