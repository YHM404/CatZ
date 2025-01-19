use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self as crossterm_event, Event, KeyCode};
use std::{collections::HashMap, time::Duration};
use sysinfo::System;

mod app;
mod args;
mod event;
mod terminal;
mod ui;
pub mod utils;

use crate::{app::App, args::Args, terminal::TerminalHandler};

fn run(args: Args) -> Result<()> {
    let mut terminal = TerminalHandler::new()?;
    terminal::setup_panic_hook()?;

    let mut app = App::new(args.interval);
    let mut sys = System::new_all();
    let mut last_cpu_values = HashMap::new();

    loop {
        terminal.terminal.draw(|f| {
            ui::ui(f, &app);
            if app.save_prompt {
                ui::draw_save_prompt(f, &app);
            }
        })?;

        if crossterm_event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = crossterm_event::read()? {
                event::handle_key_events(key.code, &mut app, &sys)?;
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

    terminal.cleanup()?;
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();
    run(args)
}
