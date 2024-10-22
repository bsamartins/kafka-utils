use crate::cmd::table;
use clap::{Args, Subcommand};
use common::kafka;
use common::kafka::client::Config;
use common::kafka::types::ListTopicEntry;
use rdkafka::admin::AdminOptions;
use std::borrow::Cow;
use tabled::Tabled;

pub fn list_topics_cmd(config: &Config) {
    println!("Listing topics");

    let topics: Vec<ListTopicTable> = kafka::topic::list_topics(config)
        .iter().map(|e| ListTopicTable(e.clone()))
        .collect::<Vec<_>>();

    let table = table::create(topics)
        .to_string();

    println!("{table}")
}

pub async fn delete_topics_cmd(config: &Config, run: bool, topic_name: Option<String>) {
    let topics = kafka::topic::list_topics_names(config);
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
        let result = kafka::client::create_admin_client(config).delete_topics(&delete_topics, &admin_options).await;
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

struct ListTopicTable(ListTopicEntry);

impl Tabled for ListTopicTable {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.name.as_str().into(),
            self.0.partitions.to_string().into(),
            self.0.replication_factor.to_string().into(),
            self.0.message_count.to_string().into(),
            self.0.size.to_string().into(),
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

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct TopicsArgs {
    #[command(subcommand)]
    pub(crate) command: Option<TopicsCommands>,
}

#[derive(Debug, Subcommand)]
pub enum TopicsCommands {
    List,
    Delete(TopicsDeleteArgs),
}

#[derive(Debug, Args)]
pub struct TopicsListArgs {}

#[derive(Debug, Args)]
pub struct TopicsDeleteArgs {
    #[arg(short, long)]
    pub(crate) topic_name: Option<String>,
    #[arg(short, long)]
    pub(crate) run: bool,
}
