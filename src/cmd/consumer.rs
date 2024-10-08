use crate::cmd::table;
use crate::kafka::{create_base_client, IamClientContext};
use crate::types::ListedConsumerGroup;
use clap::{Args, Subcommand};
use rdkafka::consumer::Consumer;
use rdkafka::ClientConfig;
use std::borrow::Cow;
use std::time::Duration;
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
    List,
}

pub(crate) fn list(config: ClientConfig, context: IamClientContext, timeout: Duration) {
    let result = create_base_client(config, context)
        .fetch_group_list(None, timeout)
        .expect("could not fetch group list");

    let mut groups = result.groups()
        .iter()
        .map(|group|
            ListedConsumerGroup {
                name: group.name().into(),
            }
        )
        .collect::<Vec<_>>();
    groups.sort_by_key(|i| i.name.clone());

    println!("{}", table::create(groups));
}

impl Tabled for ListedConsumerGroup {
    const LENGTH: usize = 5;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name.as_str().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "Name".into(),
        ]
    }
}

