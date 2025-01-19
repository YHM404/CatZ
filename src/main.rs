use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use regex::Regex;
use std::{
    collections::HashMap,
    io::{self, stdout},
    time::{Duration, Instant},
};
use sysinfo::{Pid, Process, System};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Monitor CPU and memory usage of specific processes"
)]
struct Args {
    /// Process name patterns to monitor (supports regex)
    #[arg(required = true)]
    patterns: Vec<String>,

    /// Update interval in seconds
    #[arg(short, long, default_value = "2")]
    interval: u64,
}

struct App {
    processes: Vec<(String, f32, f64)>, // name, cpu%, memory(MB)
    patterns: Vec<Regex>,
    interval: Duration,
    last_tick: Instant,
    should_quit: bool,
    input_mode: bool,  // Whether in input mode
    input_buffer: String, // Buffer for new pattern input
}

impl App {
    fn new(patterns: Vec<Regex>, interval: u64) -> Self {
        Self {
            processes: Vec::new(),
            patterns,
            interval: Duration::from_secs(interval),
            last_tick: Instant::now(),
            should_quit: false,
            input_mode: false,
            input_buffer: String::new(),
        }
    }

    fn add_pattern(&mut self, pattern: &str) {
        if let Ok(regex) = Regex::new(pattern) {
            self.patterns.push(regex);
        }
    }

    fn remove_pattern(&mut self, pattern: &str) {
        self.patterns.retain(|p| p.as_str() != pattern);
    }

    fn update(&mut self, sys: &mut System, last_cpu_values: &mut HashMap<Pid, f32>) {
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(self.last_tick).as_millis() as u64;

        // Store current CPU times
        let mut current_cpu_values: HashMap<Pid, f32> = HashMap::new();
        for (pid, process) in sys.processes() {
            current_cpu_values.insert(*pid, process.cpu_usage());
        }

        sys.refresh_all();

        let mut processes: Vec<(String, f32, f64)> = sys
            .processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name();
                self.patterns.iter().any(|pattern| pattern.is_match(name))
            })
            .map(|(pid, process)| {
                let cpu_usage = if let (Some(old_time), Some(new_time)) =
                    (last_cpu_values.get(pid), current_cpu_values.get(pid))
                {
                    calculate_cpu_percentage(*old_time, *new_time, elapsed)
                } else {
                    process.cpu_usage()
                };
                let memory = process.memory() as f64 / 1024.0 / 1024.0;
                (process.name().to_string(), cpu_usage, memory)
            })
            .collect();

        processes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        self.processes = processes;

        *last_cpu_values = current_cpu_values;
        self.last_tick = current_time;
    }

    fn tick(&mut self) {
        self.should_quit = false;
    }
}

fn calculate_cpu_percentage(old_time: f32, new_time: f32, elapsed_ms: u64) -> f32 {
    if elapsed_ms == 0 {
        return 0.0;
    }
    let cpu_delta = new_time - old_time;
    (cpu_delta * 100.0) / elapsed_ms as f32
}

fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.size());

    // Title
    let title_text = if app.input_mode {
        format!(
            "Process Monitor - Adding pattern: {}_",
            app.input_buffer
        )
    } else {
        format!(
            "Process Monitor - Watching patterns: {} (a:add, d:delete last, q:quit)",
            app.patterns
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, main_layout[0]);

    // Process table
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
        Cell::from("CPU%").style(Style::default().fg(Color::Yellow)),
        Cell::from("Memory(MB)").style(Style::default().fg(Color::Yellow)),
    ])
    .height(1);

    let rows = app.processes.iter().map(|(name, cpu, mem)| {
        Row::new(vec![
            Cell::from(name.as_str()),
            Cell::from(format!("{:.1}", cpu)),
            Cell::from(format!("{:.1}", mem)),
        ])
        .height(1)
    });

    let table = Table::new(rows.collect::<Vec<_>>(), [
        Constraint::Percentage(60),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(table, main_layout[1]);
}

fn run(args: Args) -> Result<()> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        std::process::exit(0);
    })?;

    // Create app and run it
    let patterns = args
        .patterns
        .iter()
        .map(|p| Regex::new(p).expect("Invalid regex pattern"))
        .collect();
    let mut app = App::new(patterns, args.interval);
    let mut sys = System::new_all();
    let mut last_cpu_values = HashMap::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if app.input_mode {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Enter => {
                            if !app.input_buffer.is_empty() {
                                app.add_pattern(&app.input_buffer);
                                app.input_buffer.clear();
                            }
                            app.input_mode = false;
                        }
                        KeyCode::Esc => {
                            app.input_buffer.clear();
                            app.input_mode = false;
                        }
                        _ => {}
                    }
                } else {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('q'), _)
                        | (KeyCode::Esc, _) => {
                            app.should_quit = true;
                        }
                        (KeyCode::Char('a'), _) => {
                            app.input_mode = true;
                        }
                        (KeyCode::Char('d'), _) => {
                            if let Some(last_pattern) = app.patterns.last() {
                                let pattern = last_pattern.as_str().to_string();
                                app.remove_pattern(&pattern);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        if app.last_tick.elapsed() >= app.interval {
            app.update(&mut sys, &mut last_cpu_values);
            app.tick();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    run(args)
}
