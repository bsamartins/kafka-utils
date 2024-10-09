use crate::table::LocalTable;
use crate::test_data::Data;
use color_eyre::eyre::WrapErr;
use color_eyre::owo_colors::OwoColorize;
use convert_case::{Case, Casing};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, HighlightSpacing, List, ListItem, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Default, Clone)]
pub struct App {
    input_mode: InputMode,
    input: Input,

    commands: Vec<String>,
    command: Option<Command>,

    error: Option<String>,

    table: Option<LocalTable>,

    data: Vec<Data>,
    longest_item_lens: (u16, u16, u16),

    exit: bool,
}

#[derive(Debug, Clone)]
enum InputMode {
    DEFAULT,
    COMMAND,
}

impl Default for InputMode {
    fn default() -> Self { InputMode::DEFAULT }
}

#[derive(Debug, Clone, EnumIter, IntoStaticStr)]
enum Command {
    ListTopics
}

impl Command {
    fn from(s: String) -> Option<Command> {
        Command::iter().find(|e| {
            let e_str: &str = e.into();
            e_str.to_case(Case::Kebab) == s
        })
    }
}

impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => self.exit(),
            _ => {
                match self.input_mode {
                    InputMode::COMMAND => {
                        match key_event.code {
                            KeyCode::Esc => {
                                if !self.has_error() {
                                    self.input_mode = InputMode::DEFAULT;
                                    self.input.reset();
                                }
                                self.clear_error()
                            },
                            KeyCode::Enter => {
                                if !self.has_error() {
                                    self.execute_command();
                                }
                            }
                            _ => {
                                if !self.has_error() {
                                    self.input.handle_event(&Event::Key(key_event));
                                }
                            }
                        }
                    }
                    InputMode::DEFAULT => {
                        match key_event.code {
                            KeyCode::Char('q') => self.exit(),
                            KeyCode::Char(':') => self.input_mode = InputMode::COMMAND,
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_command(&mut self) {
        let command = Command::from(self.input.to_string());
        match command {
            Some(cmd) => {
                self.commands.push(self.input.to_string());
                self.command = Some(cmd);
                self.input.reset();
                self.input_mode = InputMode::DEFAULT;
                self.table = Some(LocalTable::new());
                self.clear_error();
            }
            _ => {
                self.set_error_message(format!("Unknown command '{}'", self.input));
            }
        }
    }

    fn set_error_message(&mut self, message: String) {
        self.error = Some(message);
    }
    fn clear_error(&mut self) {
        self.error = None;
    }

    fn has_error(&self) -> bool { self.error.is_some() }

    fn exit(&mut self) {
        self.exit = true;
    }

    // fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
    //     frame.render_stateful_widget(
    //         Scrollbar::default()
    //             .orientation(ScrollbarOrientation::VerticalRight)
    //             .begin_symbol(None)
    //             .end_symbol(None),
    //         area.inner(Margin {
    //             vertical: 1,
    //             horizontal: 1,
    //         }),
    //         &mut self.table.scroll_state,
    //     );
    // }
    //
    // fn render_footer(&self, frame: &mut Frame, area: Rect) {
    //     let info_footer = Paragraph::new(Line::from("footer"))
    //         .style(
    //             Style::new()
    //                 .fg(self.colors.row_fg)
    //                 .bg(self.colors.buffer_bg),
    //         )
    //         .centered()
    //         .block(
    //             Block::bordered()
    //                 .border_type(BorderType::Double)
    //                 .border_style(Style::new().fg(self.colors.footer_border_color)),
    //         );
    //     frame.render_widget(info_footer, area);
    // }

    fn render_command_view(&self, cmd: &Command, block: Block, area: Rect, buf: &mut Buffer) {
        match cmd {
            Command::ListTopics => {
                self.render_table(block, area, buf);
            }
        }
    }

    fn render_table(&self, block: Block, area: Rect, buf: &mut Buffer) {
        let table = self.clone().table.unwrap();
        let header_style = Style::default()
            .fg(table.colors.header_fg)
            .bg(table.colors.header_bg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(table.colors.selected_style_fg);

        let header = ["Name", "Address", "Email"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = self.data.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => table.colors.normal_row_color,
                _ => table.colors.alt_row_color,
            };
            let item = data.ref_array();
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
                .collect::<Row>()
                .style(Style::new().fg(table.colors.row_fg).bg(color))
                .height(4)
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2),
            ],
        )
            .header(header)
            .highlight_style(selected_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(table.colors.buffer_bg)
            .highlight_spacing(HighlightSpacing::Always)
            .block(block);

        t.render(area, buf)
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input_size = match self.input_mode {
            InputMode::COMMAND => 3,
            _ => 0
        };

        let vertical = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(input_size),
            Constraint::Min(1),
        ]);

        let [_, input_area, main_area] = vertical.areas(area);

        Paragraph::new(self.input.value())
            .style(match self.input_mode {
                InputMode::DEFAULT => Style::default(),
                InputMode::COMMAND => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().title("Input").borders(Borders::ALL))
            .render(input_area, buf);

        let messages: Vec<ListItem> = self
            .commands
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();

        let mut main_block = Block::bordered();
        main_block = match &self.command {
            Some(cmd) => {
                let enum_str: &str = cmd.into();
                main_block.title(enum_str)
            },
            None => main_block
        };

        match &self.command {
            Some(cmd) => {
                let table = &self.table;
                self.render_command_view(cmd, main_block, main_area, buf)
            },
            None => {
                List::new(messages)
                    .block(main_block)
                    .render(main_area, buf);
            }
        }

        if self.has_error() {
            let message = self.error.clone().unwrap();
            let block = Block::bordered().title("error");

            let pop_area = popup_area(area, 60, 20);

            Paragraph::new(message)
                .style(Color::Red)
                .block(block)
                .render(pop_area, buf);
        }
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
