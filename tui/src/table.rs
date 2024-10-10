use ratatui::style::palette::tailwind::{Palette, SLATE};
use ratatui::style::Color;
use ratatui::widgets::TableState;

#[derive(Debug, Clone)]
pub struct LocalTable {
    pub(crate) colors: TableColors,
    pub(crate) state: TableState,
    pub(crate) definition: TableDefinition,
}

impl LocalTable {
    pub(crate) fn new() -> Self {
        Self {
            colors: TableColors::new(&SLATE),
            state: TableState::default(),
            definition: TableDefinition::empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableColors {
    pub(crate) buffer_bg: Color,
    pub(crate) header_bg: Color,
    pub(crate) header_fg: Color,
    pub(crate) row_fg: Color,
    pub(crate) selected_style_fg: Color,
    pub(crate) normal_row_color: Color,
    pub(crate) alt_row_color: Color,
}

impl TableColors {
    const fn new(color: &Palette) -> Self {
        Self {
            buffer_bg: SLATE.c950,
            header_bg: color.c900,
            header_fg: SLATE.c200,
            row_fg: SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: SLATE.c950,
            alt_row_color: SLATE.c900,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableDefinition {
    pub(crate) headers: Vec<String>,
}

impl TableDefinition {
    pub(crate) fn new(headers: Vec<&str>) -> Self {
        Self {
            headers: headers.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn empty() -> Self {
        Self {
            headers: vec![]
        }
    }
}