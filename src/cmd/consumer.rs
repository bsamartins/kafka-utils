use crate::cmd::table;
use crate::kafka::{create_base_client, IamClientContext};
use crate::types::ListedConsumerGroup;
use clap::{Args, Subcommand};
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;
use std::borrow::Cow;
use std::time::Duration;
use rdkafka::groups::GroupInfo;
use tabled::Tabled;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ConsumerArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ConsumerCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConsumerCommands {
    List(ListConsumerArgs),
}

#[derive(Debug, Args)]
pub struct ListConsumerArgs {
    #[arg(short, long)]
    pub(crate) consumer_name: Option<String>,
}

pub(crate) fn list(config: ClientConfig, context: IamClientContext, timeout: Duration, group_query: Option<String>) {
    let result = create_base_client(config, context)
        .fetch_group_list(None, timeout)
        .expect("could not fetch group list");

    let mut groups = result.groups()
        .iter()
        .filter(|g| filter_group(g, group_query.clone()))
        .map(|group|
            ListedConsumerGroup {
                name: group.name().into(),
                state: group.state().into(),
            }
        )
        .collect::<Vec<_>>();
    groups.sort_by_key(|i| i.name.clone());

    println!("{}", table::create(groups));
}

fn filter_group(group: &GroupInfo, group_query: Option<String>) -> bool {
    match group_query.clone() {
        Some(q) => group.name().starts_with(q.as_str()),
        None => true,
    }
}

impl Tabled for ListedConsumerGroup {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name.as_str().into(),
            self.state.as_str().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "Name".into(),
            "State".into(),
        ]
    }
}

