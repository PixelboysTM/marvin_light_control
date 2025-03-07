use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[allow(dead_code)]
pub async fn create_tui() {
    log::info!("Starting Ratatui...");
    let mut terminal = ratatui::init();
    let app_result = TuiApp { exit: false }.run(&mut terminal);
    ratatui::restore();
    app_result.unwrap();
}

#[derive(Debug)]
pub struct TuiApp {
    exit: bool,
}

impl TuiApp {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match &key_event.code {
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit = true
            }
            _ => {}
        }
    }
}

impl Widget for &TuiApp {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(vec![
            " ".into(),
            "Marvin".red(),
            " ".into(),
            "Light".green(),
            " ".into(),
            "Control".blue(),
            " ".into(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(Line::from("Ctrl + C to exit".underlined()))
            .border_set(border::ROUNDED);

        Paragraph::new(Line::from("Hello"))
            .block(block)
            .render(area, buf);
    }
}
