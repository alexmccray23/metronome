use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::Duration;
use crate::state::{AtomicMetronomeState, MetronomeState};
use crate::tap_tempo::TapTempo;

pub struct AppState {
    current_bpm: f64,
    state: MetronomeState,
    tap_tempo: TapTempo,
    input_mode: bool,
    input_buffer: String,
}

impl AppState {
    fn handle_key_event(
        &mut self,
        bpm_shared: &Arc<Mutex<f64>>,
        state: &AtomicMetronomeState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if self.input_mode {
                    self.handle_input_mode(key, bpm_shared);
                } else {
                    self.handle_normal_mode(key, bpm_shared, state);
                }
            }
        }
        Ok(())
    }

    fn handle_normal_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
        bpm_shared: &Arc<Mutex<f64>>,
        state: &AtomicMetronomeState,
    ) {
        match key.code {
            KeyCode::Char('k' | 'K') => {
                let mut bpm = bpm_shared.lock().unwrap();
                *bpm += 1.0;
                self.current_bpm = *bpm;
            }
            KeyCode::Char('j' | 'J') => {
                let mut bpm = bpm_shared.lock().unwrap();
                if *bpm > 1.0 {
                    *bpm -= 1.0;
                    self.current_bpm = *bpm;
                }
            }
            KeyCode::Char('q' | 'Q') => {
                self.state = MetronomeState::Stopped;
                state.store(MetronomeState::Stopped, Ordering::SeqCst);
            }
            KeyCode::Char(' ') => {
                let current_state = state.load(Ordering::SeqCst);
                let new_state = match current_state {
                    MetronomeState::Running => MetronomeState::Paused,
                    MetronomeState::Paused => MetronomeState::Running,
                    MetronomeState::Stopped => MetronomeState::Stopped,
                };
                state.store(new_state, Ordering::SeqCst);
                self.state = new_state;
            }
            KeyCode::Char('g' | 'G') => {
                if let Some(bpm) = self.tap_tempo.tap() {
                    {
                        let mut shared_bpm = bpm_shared.lock().unwrap();
                        *shared_bpm = bpm;
                    }
                    self.current_bpm = bpm;
                }
            }
            KeyCode::Char('i' | 'I') | KeyCode::Enter => {
                self.input_mode = true;
                self.input_buffer.clear();
            }
            _ => {}
        }
    }

    fn handle_input_mode(
        &mut self,
        key: crossterm::event::KeyEvent,
        bpm_shared: &Arc<Mutex<f64>>,
    ) {
        match key.code {
            KeyCode::Enter => {
                if let Ok(bpm) = self.input_buffer.parse::<f64>() {
                    if bpm > 0.0 {
                        {
                            let mut shared_bpm = bpm_shared.lock().unwrap();
                            *shared_bpm = bpm;
                        }
                        self.current_bpm = bpm;
                    }
                }
                self.input_mode = false;
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.input_mode = false;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }
}

pub async fn run(
    bpm_shared: Arc<Mutex<f64>>,
    state: Arc<AtomicMetronomeState>,
    start_bpm: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState {
        current_bpm: start_bpm,
        state: state.load(Ordering::SeqCst),
        tap_tempo: TapTempo::new(),
        input_mode: false,
        input_buffer: String::new(),
    };

    while app_state.state != MetronomeState::Stopped {
        terminal.draw(|f| {
            let chunks = if app_state.input_mode {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref())
                    .split(f.area())
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.area())
            };

            let paused_text = if app_state.state == MetronomeState::Paused {
                " [PAUSED]".red()
            } else {
                "".into()
            };

            let tap_text = if app_state.tap_tempo.is_tapping() {
                format!(" [TAP: {}]", app_state.tap_tempo.get_tap_count()).yellow()
            } else {
                "".into()
            };

            let bpm_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("{:.2}", app_state.current_bpm),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" BPM  "),
                    paused_text,
                    tap_text,
                ]),
            ];

            let bpm_block = Paragraph::new(bpm_text).centered().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Metronome ".blue().bold()).centered()),
            );
            f.render_widget(bpm_block, chunks[0]);

            // Render input field if in input mode
            if app_state.input_mode {
                let input_text = vec![
                    Line::from(""),
                    Line::from(vec![
                        "Enter BPM: ".into(),
                        Span::styled(
                            &app_state.input_buffer,
                            Style::default().fg(Color::Yellow),
                        ),
                        "_".yellow(),
                    ]),
                ];

                let input_block = Paragraph::new(input_text).centered().block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(" Input BPM (Enter to confirm, Esc to cancel) ".cyan().bold()).centered()),
                );
                f.render_widget(input_block, chunks[1]);
            }

            let controls_text = vec![
                Line::from(vec![
                    "Decrease BPM: ".into(),
                    "<J>".blue(),
                    " Increase BPM: ".into(),
                    "<K>".blue(),
                    " Pause/Resume: ".into(),
                    "<Space>".blue(),
                    " Quit: ".into(),
                    "<Q>".blue(),
                ]).centered(),
                Line::from(vec![
                    "Tap Tempo: ".into(),
                    "<G>".blue(),
                    " Manual Input: ".into(),
                    "<I>".blue(),
                ]).centered(),
            ];

            let controls_block = Paragraph::new(controls_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Controls ".yellow().bold()).centered()),
            );
            let controls_chunk_index = if app_state.input_mode { 2 } else { 1 };
            f.render_widget(controls_block, chunks[controls_chunk_index]);
        })?;

        if let Ok(new_bpm) = bpm_shared.lock() {
            app_state.current_bpm = *new_bpm;
        }

        app_state.state = state.load(Ordering::SeqCst);
        app_state.handle_key_event(&bpm_shared, &state)?;
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
