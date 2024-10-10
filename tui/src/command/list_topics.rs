use crate::table::{constraint_len_calculator, TableData, TableDefinition};
use common::kafka;
use common::kafka::client::Config;
use common::kafka::types::ListTopicEntry;
use ratatui::layout::Constraint;
use ratatui::prelude::{Alignment, Modifier, Style, Stylize, Text};
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
            Cell::from(Text::from("Partitions").alignment(Alignment::Right)),
            Cell::from(Text::from("Replication Factor").alignment(Alignment::Right)),
            Cell::from(Text::from("Message Count").alignment(Alignment::Right)),
            Cell::from(Text::from("Size").alignment(Alignment::Right)),
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
            let row = Row::new(
                vec![
                    Cell::from(r.clone().name).style(Style::new()),
                    Cell::from(Text::from(r.partitions.to_string()).alignment(Alignment::Right)),
                    Cell::from(Text::from(r.replication_factor.to_string()).alignment(Alignment::Right)),
                    Cell::from(Text::from(r.message_count.to_string()).alignment(Alignment::Right)),
                    Cell::from(Text::from(r.size.to_string()).alignment(Alignment::Right)),
                ]
            );
            if r.name.starts_with("_") {
                row.add_modifier(Modifier::DIM)
            } else {
                row
            }
        }).collect(),
        vec![
            // + 1 is for padding.
            Constraint::Fill(1),
            Constraint::Min(longest_partitions + 1),
            Constraint::Min(longest_replication_factor + 1),
            Constraint::Min(longest_message_count + 1),
            Constraint::Min(longest_size),
        ]
    )
}
