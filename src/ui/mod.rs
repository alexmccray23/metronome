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

pub struct AppState {
    current_bpm: f64,
    state: MetronomeState,
}

impl AppState {
    fn handle_key_event(
        &mut self,
        bpm_shared: &Arc<Mutex<f64>>,
        state: &AtomicMetronomeState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('k') => {
                        let mut bpm = bpm_shared.lock().unwrap();
                        *bpm += 1.0;
                        self.current_bpm = *bpm;
                    }
                    KeyCode::Char('j') => {
                        let mut bpm = bpm_shared.lock().unwrap();
                        if *bpm > 1.0 {
                            *bpm -= 1.0;
                            self.current_bpm = *bpm;
                        }
                    }
                    KeyCode::Char('q') => {
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
                    _ => {}
                }
            }
        }
        Ok(())
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
    };

    while app_state.state != MetronomeState::Stopped {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.area());

            let paused_text = if app_state.state == MetronomeState::Paused {
                " [PAUSED]".red()
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
                ]),
            ];

            let bpm_block = Paragraph::new(bpm_text).centered().block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Metronome ".blue().bold()).centered()),
            );
            f.render_widget(bpm_block, chunks[0]);

            let controls_text = vec![Line::from(vec![
                "Decrease BPM: ".into(),
                "<J>".blue(),
                " Increase BPM: ".into(),
                "<K>".blue(),
                " Pause/Resume: ".into(),
                "<Space>".blue(),
                " Quit: ".into(),
                "<Q>".blue(),
            ])
            .centered()];

            let controls_block = Paragraph::new(controls_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Controls ".yellow().bold()).centered()),
            );
            f.render_widget(controls_block, chunks[1]);
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
