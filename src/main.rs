mod kafka;
mod iam;
mod types;

use crate::kafka::IamClientContext;
use crate::types::ListedTopic;
use aws_types::region::Region;
use clap::{Args, Parser, Subcommand, ValueEnum};
use rdkafka::admin::AdminOptions;
use rdkafka::ClientConfig;
use std::borrow::Cow;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::themes::Colorization;
use tabled::settings::{Alignment, Color, Settings, Style};
use tabled::{Table, Tabled};
use tokio::runtime::Handle;

const NUMERIC_SETTINGS: Settings<Alignment, Alignment> = Settings::new(Alignment::top(), Alignment::right());
fn head_color() -> Color { Color::BG_WHITE | Color::FG_BLACK }
fn odd_color() -> Color { Color::FG_WHITE }

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
        println!("[{}] {}:{}", broker.id, broker.host, broker.port)
    });
}

fn list_topics_cmd(config: ClientConfig, context: IamClientContext, timeout: u64) {
    println!("Listing topics");

    let topics = kafka::list_topics(config, context, timeout);
    let style = Style::modern()
        .remove_horizontal();

    let table = Table::new(topics.iter().clone())
        .with(style)
        .with(even_odd_rows(topics.len(), true, Color::empty(), odd_color()))
        .with(Colorization::exact([head_color()], Rows::first()))
        .modify(Columns::single(1), NUMERIC_SETTINGS)
        .modify(Columns::single(2), NUMERIC_SETTINGS)
        .modify(Columns::single(3), NUMERIC_SETTINGS)
        .modify(Columns::single(4), NUMERIC_SETTINGS)
        .to_string();

    println!("{table}")
}

async fn delete_topics_cmd(client_config: ClientConfig, context: IamClientContext, run: bool, topic_name: Option<String>, timeout: u64) {
    let topics = kafka::list_topics_names(client_config.clone(), context.clone(), timeout);
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

impl Tabled for ListedTopic {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name.as_str().into(),
            self.partitions.to_string().into(),
            self.replication_factor.to_string().into(),
            self.message_count.to_string().into(),
            self.size.to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "Name".into(),
            "Partitions".into(),
            "Replication Factor".into(),
            "Message Count".into(),
            "Size".into(),
        ]
    }
}

fn even_odd_rows(len: usize, has_header: bool, even_color: Color, odd_color: Color) -> Colorization {
    let start_row = if has_header { 1 } else { 0 };
    let end_row = if has_header { len + 1 } else { len };
    Colorization::rows(
        (start_row..end_row).map(|i| {
            if i % 2 == 0 {
                even_color.clone()
            } else {
                odd_color.clone()
            }
        })
    )
}
