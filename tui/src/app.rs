use color_eyre::eyre::WrapErr;
use convert_case::{Case, Casing};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, Default)]
pub struct App {
    input_mode: InputMode,
    input: Input,
    commands: Vec<String>,
    command: Option<Command>,
    error: Option<String>,
    exit: bool,
}

#[derive(Debug)]
enum InputMode {
    DEFAULT,
    COMMAND,
}

impl Default for InputMode {
    fn default() -> Self { InputMode::DEFAULT }
}

#[derive(Debug, EnumIter, IntoStaticStr)]
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
        match self.input_mode {
            InputMode::COMMAND => {
                match key_event.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::DEFAULT;
                        self.input.reset();
                        self.error = None;
                    },
                    KeyCode::Enter => self.execute_command(),
                    _ => {
                        self.input.handle_event(&Event::Key(key_event));
                    }
                }
            }
            InputMode::DEFAULT => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => self.exit(),
                    KeyCode::Char(':') => self.input_mode = InputMode::COMMAND,
                    _ => {}
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
                self.error = None;
            }
            _ => {
                self.error = Some(format!("Unknown command '{}'", self.input));
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
                render_command_view(cmd, main_block)
            },
            None => {
                List::new(messages)
                    .block(main_block)
                    .render(main_area, buf);
            }
        }

        let error= &self.error;
        if error.is_some() {
            let message = error.clone().unwrap();
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

fn render_command_view(cmd: &Command, block: Block) {
    todo!()
}