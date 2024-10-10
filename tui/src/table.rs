use ratatui::prelude::Constraint;
use ratatui::style::palette::tailwind::CYAN;
use ratatui::style::Color;
use ratatui::widgets::{Cell, Row, TableState};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct LocalTable<'a> {
    pub(crate) colors: TableColors,
    pub(crate) state: TableState,
    pub(crate) definition: TableDefinition<'a>,
}

impl<'a> LocalTable<'a> {
    pub(crate) fn new() -> Self {
        Self {
            colors: TableColors::new(),
            state: TableState::default(),
            definition: TableDefinition::empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableColors {
    pub(crate) border: Color,
    pub(crate) header_fg: Color,
}

impl TableColors {
    const fn new() -> Self {
        Self {
            border: CYAN.c400,
            header_fg: Color::White,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableDefinition<'a> {
    pub(crate) headers: Vec<Cell<'a>>,
}

impl<'a> TableDefinition<'a> {
    pub(crate) fn new(headers: Vec<Cell<'a>>) -> Self {
        Self {
            headers,
        }
    }
    fn empty() -> Self {
        Self {
            headers: vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableData<'a> {
    pub rows: Vec<Row<'a>>,
    pub widths: Vec<Constraint>,
}

impl<'a> TableData<'a> {
    pub(crate) fn new(rows: Vec<Row<'a>>, widths: Vec<Constraint>) -> Self {
        Self {
            rows,
            widths,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            rows: Vec::new(),
            widths: Vec::new(),
        }
    }
}

pub(crate) fn constraint_len_calculator(item: &str) -> u16 {
    item.width() as u16
}
