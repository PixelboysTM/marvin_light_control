use ansi_to_tui::IntoText;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::prelude::Margin;
use ratatui::prelude::StatefulWidget;
use ratatui::text::Text;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, layout::{Constraint, Direction, Flex, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, ToLine},
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
    DefaultTerminal,
    Frame,
};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tui_logger::{ExtLogRecord, LogFormatter};

use crate::AServiceImpl;

pub async fn create_tui(
    shutdown_handler: CancellationToken,
    exit_flag: Arc<RwLock<bool>>,
    service_obj: AServiceImpl,
    log_rx: std::sync::mpsc::Receiver<Vec<u8>>,
) {
    log::info!("Starting Ratatui...");
    let mut terminal = ratatui::init();
    let app_result = TuiApp {
        shutdown_handler,
        exit_flag,
        service_obj,
        tui_state: TuiAppState {
            exit: ExitState::Idle,
            meta_information: None,
            log_state: LogWidgetState {
                rx: log_rx,
                paragraphs: Text::default(),
                scroll: 0,
                scroll_state: ScrollbarState::default(),
            },
        },
    }
    .run(&mut terminal)
    .await;
    ratatui::restore();
    app_result.unwrap();
}

pub struct TuiApp {
    shutdown_handler: CancellationToken,
    exit_flag: Arc<RwLock<bool>>,
    service_obj: AServiceImpl,
    tui_state: TuiAppState,
}

pub struct TuiAppState {
    exit: ExitState,
    meta_information: Option<MetaInformation>,
    log_state: LogWidgetState,
}

pub struct LogWidgetState {
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    paragraphs: Text<'static>,
    scroll: usize,
    scroll_state: ScrollbarState,
}

pub struct LogWidget;
pub struct MainWidget;

impl StatefulWidget for LogWidget {
    type State = LogWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered().title("LOG").border_set(border::ROUNDED);

        let remaining_height = block.inner(area).height as usize;

        state.scroll_state = state.scroll_state.content_length(
            state
                .paragraphs
                .lines
                .len()
                .saturating_sub(remaining_height),
        );

        state.scroll = state.scroll.min(state.paragraphs.lines.len()).max(0);
        state.scroll_state = state.scroll_state.position(state.scroll);

        Paragraph::new(state.paragraphs.clone())
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((state.scroll.saturating_sub(remaining_height) as u16, 0))
            .render(area, buf);

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .render(
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut state.scroll_state,
            );
    }
}

impl TuiApp {
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !matches!(self.tui_state.exit, ExitState::Quit) {
            if *self.exit_flag.read().await {
                self.tui_state.exit = ExitState::Quit;
            }

            self.update_meta_information().await;

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;

            tokio::task::yield_now().await;
        }
        Ok(())
    }

    async fn update_meta_information(&mut self) {
        if self.service_obj.project_valid().await {
            self.tui_state.meta_information = Some(MetaInformation {});
        } else {
            self.tui_state.meta_information = None;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(MainWidget, frame.area(), &mut self.tui_state);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while let Ok(event) = self.tui_state.log_state.rx.try_recv() {
            let mut s = event.into_text().unwrap();
            let len = s.iter().len();
            self.tui_state
                .log_state
                .paragraphs
                .lines
                .append(&mut s.lines);
            // for (i, line) in s.lines.into_iter().enumerate() {
            //     self.tui_state.log_state.paragraphs.lines.insert(i, line);
            // }
            self.tui_state.log_state.scroll = self.tui_state.log_state.scroll.saturating_add(len);
            self.tui_state.log_state.scroll_state = self
                .tui_state
                .log_state
                .scroll_state
                .position(self.tui_state.log_state.scroll);
        }

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
                self.tui_state.exit = match self.tui_state.exit {
                    ExitState::Idle => ExitState::UserConfirm,
                    ExitState::UserConfirm => {
                        self.shutdown_handler.cancel();
                        ExitState::Exiting
                    }
                    ExitState::Exiting => ExitState::Quit,
                    ExitState::Quit => ExitState::Quit,
                }
            }
            KeyCode::Char('y') if self.tui_state.exit == ExitState::UserConfirm => {
                self.shutdown_handler.cancel();
                self.tui_state.exit = ExitState::Exiting;
            }
            KeyCode::Char('n') if self.tui_state.exit == ExitState::UserConfirm => {
                self.tui_state.exit = ExitState::Idle;
            }
            KeyCode::Up => {
                self.tui_state.log_state.scroll = self.tui_state.log_state.scroll.saturating_sub(1);
                self.tui_state.log_state.scroll_state = self
                    .tui_state
                    .log_state
                    .scroll_state
                    .position(self.tui_state.log_state.scroll);
            }
            KeyCode::Down => {
                self.tui_state.log_state.scroll = self.tui_state.log_state.scroll.saturating_add(1);
                self.tui_state.log_state.scroll_state = self
                    .tui_state
                    .log_state
                    .scroll_state
                    .position(self.tui_state.log_state.scroll);
            }
            _ => {}
        }
    }
}

impl StatefulWidget for MainWidget {
    type State = TuiAppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
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
        // Paragraph::new(title.centered().on_white().bold()).render(layout[0], buf);

        let main_block = Block::bordered()
            .title(title.centered().bold())
            .border_set(border::ROUNDED)
            .title_bottom(Line::from("Ctrl + C to exit".underlined()))
            .border_type(ratatui::widgets::BorderType::Thick);
        let a2 = main_block.inner(area);
        main_block.render(area, buf);
        let area = a2;

        let layout = Layout::new(
            Direction::Vertical,
            [Constraint::Fill(1), Constraint::Fill(4)],
        )
        .split(area);

        let meta_block = Block::bordered().title("Meta").border_set(border::ROUNDED);

        match &state.meta_information {
            Some(_meta) => Paragraph::new("MetaInformmation tui not implemented")
                .block(meta_block)
                .render(layout[0], buf),
            None => Paragraph::new("No Project is currently loaded")
                .alignment(Alignment::Center)
                .block(meta_block)
                .render(layout[0], buf),
        }

        LogWidget.render(layout[1], buf, &mut state.log_state);

        if matches!(state.exit, ExitState::UserConfirm) {
            let mut btns = Line::default();
            btns.push_span("[YES (y)]");
            btns.push_span("-");
            btns.push_span("[NO (n)]");

            let block = Block::bordered().title_bottom(btns);
            let area = popup_area(area, 30, 20);
            Clear.render(area, buf);
            Paragraph::new(
                "Are u sure you want to quit Marvin Light Control?"
                    .to_line()
                    .centered(),
            )
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
        }
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
