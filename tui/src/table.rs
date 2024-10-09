use ratatui::style::palette::tailwind;
use ratatui::style::Color;
use ratatui::style::palette::tailwind::{Palette, SLATE};

#[derive(Debug, Clone)]
pub struct LocalTable {
    pub(crate) colors: TableColors,
}

impl LocalTable {
    pub(crate) fn new() -> Self {
        Self {
            colors: TableColors::new(&SLATE),
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
    footer_border_color: Color,
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
            footer_border_color: color.c400,
        }
    }
}
