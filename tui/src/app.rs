use color_eyre::eyre::{bail, WrapErr};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    input_mode: InputMode,
    input: Input,
    commands: Vec<String>,
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
                    },
                    KeyCode::Enter => self.submit_message(),
                    _ => {
                        self.input.handle_event(&Event::Key(key_event));
                    }
                }
            }
            InputMode::DEFAULT => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char(':') => self.input_mode = InputMode::COMMAND,
                    KeyCode::Left => self.decrement_counter()?,
                    KeyCode::Right => self.increment_counter()?,
                    KeyCode::Up => self.decrement_counter()?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn submit_message(&mut self) {
        self.commands.push(self.input.to_string());
        self.input.reset();
        self.input_mode = InputMode::DEFAULT;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) -> color_eyre::Result<()> {
        self.counter += 1;
        if self.counter > 2 {
            bail!("counter overflow");
        }
        Ok(())
    }

    fn decrement_counter(&mut self) -> color_eyre::Result<()> {
        self.counter -= 1;
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = Title::from(" Counter App Tutorial ".bold());
        // let instructions = Title::from(Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        // ]));
        // let block = Block::bordered()
        //     .title(title.alignment(Alignment::Center))
        //     .title(
        //         instructions
        //             .alignment(Alignment::Center)
        //             .position(Position::Bottom),
        //     )
        //     .border_set(border::THICK);

        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);

        // Paragraph::new(counter_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
        ]);

        let [input_area, main_area] = vertical.areas(area);

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
        List::new(messages)
            .block(Block::bordered().title("Main"))
            .render(main_area, buf);
    }
}