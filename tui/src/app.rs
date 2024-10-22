use crate::command;
use crate::command::list_topics::ListTopicsState;
use crate::table::{LocalTable, TableData};
use color_eyre::eyre::WrapErr;
use common::kafka;
use common::kafka::client::Config;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Flex;
use ratatui::prelude::{Alignment, Buffer, Color, Constraint, Layout, Modifier, Rect, Style, Stylize, Widget};
use ratatui::style::Styled;
use ratatui::widgets::{Block, Borders, Cell, Clear, HighlightSpacing, Padding, Paragraph, Row, Table};
use ratatui::{DefaultTerminal, Frame};
use std::ops::Deref;
use std::string::ToString;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Clone)]
pub struct App<'a> {
    pub(crate) config: Config,

    input_mode: InputMode,
    input: Input,

    command: Box<Command>,

    error: Option<String>,

    pub(crate) table: LocalTable<'a>,
    data: TableData<'a>,

    exit: bool,
    popup_type: PopupType,
}

#[derive(Debug, Clone)]
enum InputMode {
    DEFAULT,
    COMMAND,
}

impl Default for InputMode {
    fn default() -> Self { InputMode::DEFAULT }
}

#[derive(Debug, Clone)]
enum Command {
    None,
    ListTopics(ListTopicsState)
}

impl Command {

    const CMD_LIST_TOPICS: &'static str = "list-topics";

    fn parse(s: String) -> Option<Command> {
        match s.as_str() {
            Command::CMD_LIST_TOPICS => Some(Command::ListTopics(ListTopicsState::default())),
            _ => None
        }
    }

    fn name(self) -> String {
        match self {
            Command::ListTopics(_) => Command::CMD_LIST_TOPICS.to_string(),
            Command::None => "none".to_string(),
        }
    }
}

impl<'a> App<'a> {

    pub fn new(config: Config) -> Self {
        Self {
            config,
            input_mode: Default::default(),
            input: Default::default(),
            command: Box::new(Command::None),
            error: None,
            table: LocalTable::new(),
            data: TableData::empty(),
            exit: false,
            popup_type: PopupType::SUCCESS,
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()
                .await
                .wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(self.clone(), frame.area(), self);
    }

    async fn handle_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .await
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(())
        }
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => self.exit(),
            _ => {
                if self.is_open() {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.close()
                        },
                        _ => {}
                    }
                    return Ok(())
                }
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
                                match self.command.deref() {
                                    Command::None => {}
                                    _ => {
                                        match key_event.code {
                                            KeyCode::Up => {
                                                self.table.state.select_previous();
                                            }
                                            KeyCode::Down => {
                                                self.table.state.select_next();
                                            }
                                            KeyCode::Char(' ') => {
                                                self.table.toggle_selected();
                                            }
                                            _ => {}
                                        }
                                        match self.command.deref() {
                                            Command::ListTopics(state) => {
                                                command::list_topics::handle_key_event(key_event, self, state.to_owned())
                                                    .await;
                                            }
                                            Command::None => {}
                                        }
                                    }
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
        match Command::parse(self.input.to_string()) {
            Some(cmd) => {
                let mut cmd_ref = Box::new(cmd);
                self.input.reset();
                self.input_mode = InputMode::DEFAULT;
                self.clear_error();
                match *cmd_ref {
                    Command::ListTopics(ref mut state) => {
                        self.table.definition = command::list_topics::create_list_topics_table_definition();
                        let topics = kafka::topic::list_topics(&self.config);
                        state.set_topics(topics.clone());
                        self.data = command::list_topics::table_from(topics)
                    }
                    Command::None => {}
                }
                self.command = cmd_ref;
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
            Command::ListTopics(_) => {
                self.draw_table(area, buf, state);
            }
            _ => {}
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
            .header
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let rows: Vec<Row> = if table_definition.selectable {
            table_data.rows.iter().enumerate().map(|(i, row)| {
                if state.table.selected.contains(&i) {
                    let style = Style::from(row.style())
                        .add_modifier(Modifier::BOLD);
                    row.clone().set_style(style)
                } else {
                    row.clone()
                }
            }).collect::<Vec<_>>()
        } else { table_data.rows };

        let t = Table::new(rows, table_data.widths)
            .header(header)
            .row_highlight_style(selected_style)
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

        let main_block = Block::bordered()
            .style(Style::new().fg(self.table.colors.border))
            .padding(Padding {
                left: 1,
                right: 1,
                ..Default::default()
            })
            .title(self.command.clone().name())
            .title_alignment(Alignment::Center);

        main_block
            .clone()
            .render(main_area, buf);

        self.render_command_view(&self.command, main_block.inner(main_area), buf, state);

        if self.has_error() {
            let message = self.error.clone().unwrap();

            let pop_area = popup_area(area, 60, 20);
            Clear.render(pop_area, buf);
            let (title, color) = match self.popup_type {
                PopupType::ERROR => ("error", Color::Red),
                PopupType::SUCCESS => ("success", Color::Green),
            };
            let block = Block::bordered()
                .title(title)
                .title_alignment(Alignment::Center)
                .title_style(Style::default())
                .bg(Color::Black);
            Paragraph::new(message)
                .style(color)
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

pub trait PopupWidget {
    fn open(&mut self, popup_type: PopupType, message: String);
    fn close(&mut self);
    fn is_open(&self) -> bool;
}

impl PopupWidget for App<'_> {
    fn open(&mut self, popup_type: PopupType, message: String) {
        self.popup_type = popup_type;
        self.set_error_message(message);
    }

    fn close(&mut self) {
        self.clear_error();
    }

    fn is_open(&self) -> bool {
        self.has_error()
    }
}

#[derive(Clone)]
pub enum PopupType {
    ERROR,
    SUCCESS
}
