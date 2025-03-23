use std::{io, sync::Arc, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tui_logger::{ExtLogRecord, LogFormatter};

use crate::AServiceImpl;

pub async fn create_tui(
    shutdown_handler: CancellationToken,
    exit_flag: Arc<RwLock<bool>>,
    service_obj: AServiceImpl,
) {
    log::info!("Starting Ratatui...");
    let mut terminal = ratatui::init();
    let app_result = TuiApp {
        exit: ExitState::Idle,
        shutdown_handler,
        exit_flag,
        meta_information: None,
        service_obj,
    }
    .run(&mut terminal)
    .await;
    ratatui::restore();
    app_result.unwrap();
}

pub struct TuiApp {
    exit: ExitState,
    shutdown_handler: CancellationToken,
    exit_flag: Arc<RwLock<bool>>,
    meta_information: Option<MetaInformation>,
    service_obj: AServiceImpl,
}

impl TuiApp {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !matches!(self.exit, ExitState::Quit) {
            if *self.exit_flag.read().await {
                self.exit = ExitState::Quit;
            }

            self.update_meta_information().await;

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;

            tokio::task::yield_now().await;
        }
        Ok(())
    }

    async fn update_meta_information(&mut self) {
        let obj = self.service_obj.read().await;
        if *obj.valid_project.read().await {
            self.meta_information = Some(MetaInformation {});
        } else {
            self.meta_information = None;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(Duration::from_millis(250))? {
            return Ok(());
        }
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
                self.exit = match self.exit {
                    ExitState::Idle => ExitState::UserConfirm,
                    ExitState::UserConfirm => {
                        self.shutdown_handler.cancel();
                        ExitState::Exiting
                    }
                    ExitState::Exiting => ExitState::Quit,
                    ExitState::Quit => ExitState::Quit,
                }
            }
            KeyCode::Char('y') if self.exit == ExitState::UserConfirm => {
                self.shutdown_handler.cancel();
                self.exit = ExitState::Exiting;
            }
            KeyCode::Char('n') if self.exit == ExitState::UserConfirm => {
                self.exit = ExitState::Idle;
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
            "Marvin".red(),
            " ".into(),
            "Light".green(),
            " ".into(),
            "Control".blue(),
        ]);

        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Fill(4),
            ],
        )
        .split(area);

        Paragraph::new(title.centered().underlined().bold()).render(layout[0], buf);

        let block = Block::bordered()
            .title("LOG")
            .title_bottom(Line::from("Ctrl + C to exit".underlined()))
            .border_set(border::ROUNDED);

        tui_logger::TuiLoggerWidget::default()
            .block(block)
            .formatter(Box::new(TuiLogFormatter))
            .render(layout[2], buf);

        if matches!(self.exit, ExitState::UserConfirm) {
            let mut btns = Line::default();
            btns.push_span("[YES (y)]".on_green().black());
            btns.push_span("-");
            btns.push_span("[NO (n)]".on_green().black());

            let block = Block::bordered().title_bottom(btns);
            let area = popup_area(area, 50, 40);
            Clear.render(area, buf);
            Paragraph::new("Are u sure you want to quit Marvin Light Control?")
                .block(block)
                .render(area, buf);
        }
    }
}

struct TuiLogFormatter;
impl LogFormatter for TuiLogFormatter {
    fn min_width(&self) -> u16 {
        4
    }
    fn format(&self, _width: usize, evt: &ExtLogRecord) -> Vec<Line> {
        let mut line = Line::from("[");

        line.push_span(format!("{} ", evt.timestamp.format("%H:%M.%S")));

        line.push_span(match evt.level {
            log::Level::Error => "ERROR".red(),
            log::Level::Warn => "WARN ".yellow(),
            log::Level::Info => "INFO ".green(),
            log::Level::Debug => "DEBUG".blue(),
            log::Level::Trace => "TRACE".gray(),
        });

        #[cfg(debug_assertions)]
        if let Some(m) = evt.module_path() {
            line.push_span(format!(
                " {}:{}",
                m,
                evt.line.map(|i| i as i64).unwrap_or(-1)
            ));
        }

        line.push_span("] ");

        line.push_span(evt.msg().to_string());

        vec![line]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExitState {
    Idle,
    UserConfirm,
    Exiting,
    Quit,
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

#[derive(Debug)]
struct MetaInformation {}
