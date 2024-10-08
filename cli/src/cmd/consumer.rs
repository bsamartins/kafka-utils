use crate::cmd::table;
use clap::{Args, Subcommand};
use common::kafka;
use common::kafka::client::Config;
use common::kafka::types::ListConsumerGroupEntry;
use std::borrow::Cow;
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

pub(crate) fn list(config: Config, consumer_group: Option<String>) {
    let groups:Vec<ListConsumerGroupEntryTable> = kafka::group::list(config, consumer_group)
        .iter()
        .map(|group| ListConsumerGroupEntryTable(group.to_owned()))
        .collect();

    println!("{}", table::create(groups));
}

pub(crate) async fn delete(config: Config, consumer_group: Option<String>) {
    let result = kafka::group::delete(config, consumer_group).await;
    result.iter().for_each(|res| println!("{:?}", res))
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

