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
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::time::Duration;

pub struct AppState {
    current_bpm: f64,
    is_running: bool,
    is_paused: bool,
}

impl AppState {
    fn handle_key_event(
        &mut self,
        bpm_shared: &Arc<Mutex<f64>>,
        running: &AtomicBool,
        paused: &AtomicBool,
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
                        self.is_running = false;
                        running.store(false, Ordering::SeqCst);
                    }
                    KeyCode::Char(' ') => {
                        let new_paused = !paused.load(Ordering::SeqCst);
                        paused.store(new_paused, Ordering::SeqCst);
                        self.is_paused = new_paused;
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
    running: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    start_bpm: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState {
        current_bpm: start_bpm,
        is_running: true,
        is_paused: paused.load(Ordering::SeqCst),
    };

    while app_state.is_running {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.area());

            let paused_text = if app_state.is_paused {
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

        app_state.handle_key_event(&bpm_shared, &running, &paused)?;
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
