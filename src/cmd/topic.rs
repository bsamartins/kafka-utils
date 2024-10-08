use std::borrow::Cow;
use clap::{Args, Subcommand};
use rdkafka::admin::AdminOptions;
use rdkafka::ClientConfig;
use tabled::settings::{Color, Style};
use tabled::settings::object::{Columns, Rows};
use tabled::settings::themes::Colorization;
use tabled::{Table, Tabled};
use crate::cmd::table;
use crate::kafka;
use crate::kafka::IamClientContext;
use crate::types::ListedTopic;

pub fn list_topics_cmd(config: ClientConfig, context: IamClientContext, timeout: u64) {
    println!("Listing topics");

    let topics = kafka::list_topics(config, context, timeout);
    let style = Style::modern()
        .remove_horizontal();

    let table = Table::new(topics.iter().clone())
        .with(style)
        .with(table::even_odd_rows(topics.len(), true, Color::empty(), table::odd_color()))
        .with(Colorization::exact([table::head_color()], Rows::first()))
        .modify(Columns::single(1), table::NUMERIC_SETTINGS)
        .modify(Columns::single(2), table::NUMERIC_SETTINGS)
        .modify(Columns::single(3), table::NUMERIC_SETTINGS)
        .modify(Columns::single(4), table::NUMERIC_SETTINGS)
        .to_string();

    println!("{table}")
}

pub async fn delete_topics_cmd(client_config: ClientConfig, context: IamClientContext, run: bool, topic_name: Option<String>, timeout: u64) {
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
