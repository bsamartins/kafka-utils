use crate::cmd::table;
use clap::{Args, Subcommand};
use common::kafka;
use common::kafka::client::Config;
use common::kafka::types::ListBrokerEntry;
use std::borrow::Cow;
use tabled::Tabled;

pub fn list_brokers_cmd(config: Config) {
    let brokers = kafka::broker::list_brokers(config)
        .iter().map(|e| ListBrokerTable(e.clone()))
        .collect();
    println!("{}", table::create(brokers))
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct ClusterArgs {
    #[command(subcommand)]
    pub(crate) command: Option<ClusterCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ClusterCommands {
    Brokers,
}

struct ListBrokerTable(ListBrokerEntry);

impl Tabled for ListBrokerTable {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.id.to_string().into(),
            self.0.host.as_str().into(),
            self.0.port.to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "ID".into(),
            "Host".into(),
            "Port".into(),
        ]
    }
}
