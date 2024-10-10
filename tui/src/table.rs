use ratatui::prelude::Constraint;
use ratatui::style::palette::tailwind::CYAN;
use ratatui::style::Color;
use ratatui::widgets::{Cell, Row, TableState};
use std::collections::HashSet;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct LocalTable<'a> {
    pub(crate) colors: TableColors,
    pub(crate) state: TableState,
    pub(crate) definition: TableDefinition<'a>,
    pub(crate) selected: HashSet<usize>,
}

impl<'a> LocalTable<'a> {
    pub(crate) fn new() -> Self {
        Self {
            colors: TableColors::new(),
            state: TableState::default(),
            definition: TableDefinition::empty(),
            selected: HashSet::new(),
        }
    }

    pub(crate) fn toggle_selected(&mut self) {
        match self.state.selected() {
            Some(selected) => {
                if self.selected.contains(&selected) {
                    self.selected.remove(&selected);
                } else {
                    self.selected.insert(selected);
                }
            }
            None => {}
        };
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
    pub(crate) header: Vec<Cell<'a>>,
    pub(crate) selectable: bool,
}

impl<'a> TableDefinition<'a> {
    pub(crate) fn new(header: Vec<Cell<'a>>) -> Self {
        Self {
            header,
            selectable: false,
        }
    }
    fn empty() -> Self {
        Self {
            header: vec![],
            selectable: false,
        }
    }

    pub(crate) fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
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
