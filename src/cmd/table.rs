use tabled::settings::object::{Columns, Rows};
use tabled::settings::themes::Colorization;
use tabled::settings::{Alignment, Color, Settings, Style};
use tabled::{Table, Tabled};

pub(crate) const NUMERIC_SETTINGS: Settings<Alignment, Alignment> = Settings::new(Alignment::top(), Alignment::right());
pub(crate) fn head_color() -> Color { Color::BG_WHITE | Color::FG_BLACK }
pub(crate) fn odd_color() -> Color { Color::FG_WHITE }

pub(crate) fn even_odd_rows(len: usize, has_header: bool, even_color: Color, odd_color: Color) -> Colorization {
    let start_row = if has_header { 1 } else { 0 };
    let end_row = if has_header { len + 1 } else { len };
    Colorization::rows(
        (start_row..=end_row).map(|i| {
            if i % 2 == 0 {
                even_color.clone()
            } else {
                odd_color.clone()
            }
        })
    )
}

pub(crate) fn create<T: Tabled>(items: Vec<T>) -> Table {
    let style = Style::modern()
        .remove_horizontal();

    Table::new(items.iter().clone())
        .with(style)
        .with(even_odd_rows(items.len(), true, Color::empty(), odd_color()))
        .with(Colorization::exact([head_color()], Rows::first()))
        .modify(Columns::single(1), NUMERIC_SETTINGS)
        .modify(Columns::single(2), NUMERIC_SETTINGS)
        .modify(Columns::single(3), NUMERIC_SETTINGS)
        .modify(Columns::single(4), NUMERIC_SETTINGS)
        .to_owned()
}
