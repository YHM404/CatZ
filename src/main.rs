use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self as crossterm_event, Event, KeyCode};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use sysinfo::{Pid, System};

mod args;
mod config;
mod domain;
mod event;
mod terminal;
mod ui;
mod utils;

use crate::{
    args::Args,
    config::Config,
    domain::{process::ProcessInfo, state::AppState},
    terminal::TerminalHandler,
};

fn run(config: Config) -> Result<()> {
    let mut terminal = TerminalHandler::new()?;
    terminal::setup_panic_hook()?;

    let mut state = AppState::new(config.update_interval.as_secs());
    let mut sys = System::new_all();
    let mut last_cpu_values = HashMap::new();

    loop {
        terminal.terminal.draw(|f| ui::render(f, &state))?;

        if crossterm_event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm_event::read()? {
                event::handle_key_events(key.code, &mut state, &sys)?;
            }
        }

        if state.should_quit {
            break;
        }

        if state.last_tick.elapsed() >= state.interval {
            update_process_info(&mut state, &mut sys, &mut last_cpu_values);
            state.tick();
        }
    }

    terminal.cleanup()?;
    Ok(())
}

fn update_process_info(
    state: &mut AppState,
    sys: &mut System,
    last_cpu_values: &mut HashMap<Pid, f32>,
) {
    let current_time = Instant::now();
    let elapsed = current_time.duration_since(state.last_tick).as_millis() as u64;

    // Store current CPU times
    let mut current_cpu_values: HashMap<Pid, f32> = HashMap::new();
    for (pid, process) in sys.processes() {
        current_cpu_values.insert(*pid, process.cpu_usage());
    }

    sys.refresh_all();

    let monitored_processes = state.process_manager().get_monitored_processes().to_vec();
    let mut processes = Vec::new();

    for (name, pid) in monitored_processes {
        if let Some(process) = sys.process(pid) {
            let cpu_usage = if let (Some(old_time), Some(new_time)) =
                (last_cpu_values.get(&pid), current_cpu_values.get(&pid))
            {
                utils::calculate_cpu_percentage(*old_time, *new_time, elapsed)
            } else {
                process.cpu_usage()
            };
            let memory = process.memory() as f64 / 1024.0 / 1024.0;
            let process_info = ProcessInfo::new(name.clone(), pid, cpu_usage, memory);
            processes.push(process_info.clone());
            state.add_stats(process_info);
        }
    }

    state.update_processes(processes);
    *last_cpu_values = current_cpu_values;
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    let config = Config::new(args.interval);
    run(config)
}
