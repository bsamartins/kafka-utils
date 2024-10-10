use std::time::Duration;
use clap::Parser;
use common::kafka;
use common::kafka::client::Config;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "kafka-utils")]
#[command(about = "A fictional versioning CLI", long_about = None)]
pub struct Cli {
    #[arg(short, long, required = true)]
    bootstrap_servers: String,
    #[arg(short, long)]
    iam_auth: bool,
    #[arg(short, long, default_value = "10000")]
    timeout: u64,
    #[arg(short, long, default_value = "eu-west-1")]
    aws_region: String,
}

pub fn get_config(cli: Cli) -> Config {
    kafka::client::create_config(
        cli.bootstrap_servers,
        cli.iam_auth,
        cli.aws_region,
        Duration::from_millis(cli.timeout.into()),
    )
}