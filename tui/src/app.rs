use crate::command;
use crate::table::{LocalTable, TableData};
use color_eyre::eyre::WrapErr;
use common::kafka::client::Config;
use convert_case::{Case, Casing};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Flex;
use ratatui::prelude::{Alignment, Buffer, Color, Constraint, Layout, Modifier, Rect, Style, Stylize, Widget};
use ratatui::widgets::{Block, Borders, Cell, Clear, HighlightSpacing, Padding, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Clone)]
pub struct App<'a> {
    config: Config,

    input_mode: InputMode,
    input: Input,

    command: Option<Command>,

    error: Option<String>,

    table: LocalTable<'a>,
    data: TableData<'a>,

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

impl<'a> App<'a> {

    pub fn new(config: Config) -> Self {
        Self {
            config,
            input_mode: Default::default(),
            input: Default::default(),
            command: None,
            error: None,
            table: LocalTable::new(),
            data: TableData::empty(),
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(self.clone(), frame.area(), self);
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
                            _ => {
                                match self.command {
                                    Some(_) => {
                                        match key_event.code {
                                            KeyCode::Up => {
                                                self.table.state.select_previous();
                                                self.set_error_message(format!("Up - {}", self.table.state.selected().map(|v| v.to_string()).unwrap_or("None".to_string())));
                                            }
                                            KeyCode::Down => {
                                                self.table.state.select_next();
                                                self.set_error_message(format!("Down - {}", self.table.state.selected().map(|v| v.to_string()).unwrap_or("None".to_string())));
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}

                                }
                            }
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
                self.command = Some(cmd);
                self.input.reset();
                self.input_mode = InputMode::DEFAULT;
                self.clear_error();
                match cmd {
                    Command::ListTopics => {
                        self.table.definition = command::list_topics::create_list_topics_table_definition();
                        self.data = command::list_topics::list_topics(&self.config);
                    }
                }
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

    fn render_command_view(&self, cmd: &Command, area: Rect, buf: &mut Buffer, state: &mut App) {
        match cmd {
            Command::ListTopics => {
                self.draw_table(area, buf, state);
            }
        }
    }

    fn draw_table(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3)]);
        let rects = vertical.split(area);

        self.render_table(rects[0], buf, state);
    }
    fn render_table(&self, area: Rect, buf: &mut Buffer, state: &mut App) {
        let table = self.clone().table;
        let table_data = state.clone().data;
        let header_style = Style::default()
            .fg(table.colors.header_fg);
        let selected_style = Style::default()
            .add_modifier(Modifier::REVERSED);

        let table_definition = state.clone().table.definition;
        let header = table_definition
            .headers
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let t = Table::new(table_data.rows, table_data.widths)
            .header(header)
            .highlight_style(selected_style)
            .highlight_spacing(HighlightSpacing::Always);

        ratatui::prelude::StatefulWidget::render(t, area, buf, &mut state.table.state)
    }
}

impl<'a> ratatui::widgets::StatefulWidget for App<'a> {
    type State = App<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

        let mut main_block = Block::bordered()
            .style(Style::new().fg(self.table.colors.border))
            .padding(Padding {
                left: 1,
                right: 1,
                ..Default::default()
            });
        main_block = match &self.command {
            Some(cmd) => {
                let string_cmd: &str = cmd.into();
                let title = string_cmd.to_case(Case::Kebab);
                main_block.title(title)
                    .title_alignment(Alignment::Center)
            },
            None => main_block
        };

        match &self.command {
            Some(cmd) => {
                main_block
                    .clone()
                    .render(main_area, buf);
                self.render_command_view(cmd, main_block.inner(main_area), buf, state)
            },
            None => {
                main_block.render(main_area, buf);
            }
        }

        if self.has_error() {
            let message = self.error.clone().unwrap();
            let block = Block::bordered()
                .title("error")
                .title_alignment(Alignment::Center)
                .title_style(Style::default())
                .bg(Color::Black);

            let pop_area = popup_area(area, 60, 20);
            Clear.render(pop_area, buf);
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
