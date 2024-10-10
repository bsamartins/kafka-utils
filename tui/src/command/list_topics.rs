use crate::table::{constraint_len_calculator, TableData, TableDefinition};
use crate::test_data::generate_fake_names;
use ratatui::layout::Constraint;
use ratatui::widgets::{Cell, Row};
use std::cmp::max;

pub(crate) fn list_topics<'a>() -> TableData<'a> {
    table_from(generate_fake_names())
}

#[derive(Clone)]
pub struct Data {
    pub name: String,
    pub address: String,
    pub email: String,
}

impl Data {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn address(&self) -> &str {
        &self.address
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}

pub fn create_list_topics_table_definition<'a>() -> TableDefinition<'a> {
    TableDefinition::new(
        vec![
            Cell::from("Name"),
            Cell::from("Address"),
            Cell::from("Email")
        ]
    )
}

pub fn table_from<'a>(data: Vec<Data>) -> TableData<'a> {
    let mut longest_name = 0;
    let mut longest_address = 0;
    let mut longest_email = 0;

    TableData::new(
        data.iter().map(|r| {
            longest_name = max(0, constraint_len_calculator(r.name()));
            longest_address = max(0, constraint_len_calculator(r.address()));
            longest_email = max(0, constraint_len_calculator(r.email()));
            Row::new(
                vec![
                    Cell::from(r.name.clone()),
                    Cell::from(r.address.clone()),
                    Cell::from(r.email.clone()),
                ]
            )
        }).collect(),
        vec![
            // + 1 is for padding.
            Constraint::Length(longest_name + 1),
            Constraint::Min(longest_address + 1),
            Constraint::Min(longest_email),

        ]
    )
}
