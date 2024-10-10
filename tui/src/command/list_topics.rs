use crate::table::{constraint_len_calculator, TableData, TableDefinition};
use common::kafka;
use common::kafka::client::Config;
use common::kafka::types::ListTopicEntry;
use ratatui::layout::Constraint;
use ratatui::widgets::{Cell, Row};
use std::cmp::max;

pub(crate) fn list_topics<'a>(config: &Config) -> TableData<'a> {
    let topics = kafka::topic::list_topics(config.clone());
    table_from(topics)
}

pub fn create_list_topics_table_definition<'a>() -> TableDefinition<'a> {
    TableDefinition::new(
        vec![
            Cell::from("Name"),
            Cell::from("Partitions"),
            Cell::from("Replication Factor"),
            Cell::from("Message Count"),
            Cell::from("Size"),
        ]
    )
}

pub fn table_from<'a>(data: Vec<ListTopicEntry>) -> TableData<'a> {
    let mut longest_name = 0;
    let mut longest_partitions = 0;
    let mut longest_replication_factor = 0;
    let mut longest_message_count = 0;
    let mut longest_size = 0;

    TableData::new(
        data.iter().map(|r| {
            longest_name = max(longest_name, constraint_len_calculator(r.name.as_str()));
            longest_partitions = max(longest_partitions, constraint_len_calculator(r.partitions.to_string().as_str()));
            longest_replication_factor = max(longest_replication_factor, constraint_len_calculator(r.replication_factor.to_string().as_str()));
            longest_message_count = max(longest_message_count, constraint_len_calculator(r.message_count.to_string().as_str()));
            longest_size = max(longest_size, constraint_len_calculator(r.size.to_string().as_str()));
            Row::new(
                vec![
                    Cell::from(r.clone().name),
                    Cell::from(r.partitions.to_string()),
                    Cell::from(r.replication_factor.to_string()),
                    Cell::from(r.message_count.to_string()),
                    Cell::from(r.size.to_string()),
                ]
            )
        }).collect(),
        vec![
            // + 1 is for padding.
            Constraint::Length(longest_name + 1),
            Constraint::Min(longest_partitions + 1),
            Constraint::Min(longest_replication_factor + 1),
            Constraint::Min(longest_message_count + 1),
            Constraint::Min(longest_size),
        ]
    )
}
