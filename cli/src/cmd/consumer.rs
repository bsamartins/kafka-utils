use crate::cmd::table;
use common::kafka::types::ListConsumerGroupEntry;
use clap::{Args, Subcommand};
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;
use std::borrow::Cow;
use std::time::Duration;
use rdkafka::admin::AdminOptions;
use rdkafka::groups::GroupInfo;
use tabled::Tabled;
use common::kafka::client::{create_admin_client, create_base_client, IamClientContext};

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
    Delete(DeleteConsumerArgs),
}

#[derive(Debug, Args)]
pub struct ListConsumerArgs {
    #[arg(short, long)]
    pub(crate) consumer_group: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteConsumerArgs {
    #[arg(short, long)]
    pub(crate) consumer_group: Option<String>,
}

pub(crate) fn list(config: ClientConfig, context: IamClientContext, timeout: Duration, consumer_group_query: Option<String>) {
    let result = create_base_client(config, context)
        .fetch_group_list(None, timeout)
        .expect("could not fetch group list");

    let mut groups = result.groups()
        .iter()
        .filter(|g| filter_group(g, consumer_group_query.clone()))
        .map(|g|
            ListConsumerGroupEntry {
                name: g.name().into(),
                state: g.state().into(),
            }
        )
        .map(|group| ListConsumerGroupEntryTable(group))
        .collect::<Vec<_>>();
    groups.sort_by_key(|i| i.0.name.clone());

    println!("{}", table::create(groups));
}

pub(crate) async fn delete(config: ClientConfig, context: IamClientContext, timeout: Duration, consumer_group_query: Option<String>) {
    let result = create_base_client(config.clone(), context.clone())
        .fetch_group_list(None, timeout)
        .expect("could not fetch group list");

    let mut groups = result.groups()
        .iter()
        .filter(|g| filter_group(g, consumer_group_query.clone()))
        .map(|group|
            ListConsumerGroupEntry {
                name: group.name().into(),
                state: group.state().into(),
            }
        )
        .collect::<Vec<_>>();
    groups.sort_by_key(|i| i.name.clone());

    let groups_to_delete: Vec<&str>  = groups.iter().map(|g| g.name.as_str())
        .collect();

    let result = create_admin_client(config, context)
        .delete_groups(&groups_to_delete, &AdminOptions::new())
        .await
        .expect("could not delete groups");

    result.iter().for_each(|res| println!("{:?}", res))
}

fn filter_group(group: &GroupInfo, group_query: Option<String>) -> bool {
    match group_query.clone() {
        Some(q) => group.name().starts_with(q.as_str()),
        None => true,
    }
}

struct ListConsumerGroupEntryTable(ListConsumerGroupEntry);

impl Tabled for ListConsumerGroupEntryTable {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.name.as_str().into(),
            self.0.state.as_str().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "Name".into(),
            "State".into(),
        ]
    }
}

