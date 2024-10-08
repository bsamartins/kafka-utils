use tabled::settings::{Alignment, Color, Settings};
use tabled::settings::themes::Colorization;

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
