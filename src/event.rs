use crate::app::App;
use color_eyre::Result;
use crossterm::event::{Event, KeyCode};
use sysinfo::System;

pub fn handle_key_events(key_event: KeyCode, app: &mut App, sys: &System) -> Result<()> {
    match key_event {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.save_prompt {
                app.save_prompt = false;
                app.save_filename.clear();
            } else if app.input_mode || app.select_mode {
                app.cancel_input();
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('a') if !app.input_mode && !app.save_prompt => {
            app.input_mode = true;
            app.input_buffer.clear();
        }
        KeyCode::Char('d') if !app.input_mode && !app.save_prompt => {
            app.remove_selected_process();
        }
        KeyCode::Char('s') if !app.input_mode && !app.save_prompt => {
            if app.stats_mode {
                app.stats_mode = false;
                app.save_prompt = true;
                app.save_filename.clear();
            } else {
                app.stats_mode = true;
                app.stats_data.clear();
            }
        }
        KeyCode::Char(c) if app.save_prompt => {
            app.save_filename.push(c);
        }
        KeyCode::Char(c) if app.input_mode => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace if app.save_prompt => {
            app.save_filename.pop();
        }
        KeyCode::Backspace if app.input_mode => {
            app.input_buffer.pop();
        }
        KeyCode::Enter if app.save_prompt => {
            if app.save_filename.ends_with(".csv") {
                if !app.save_filename.is_empty() {
                    let _ = crate::ui::save_stats_to_csv(&app.stats_data, &app.save_filename);
                }
                app.save_prompt = false;
                app.save_filename.clear();
                app.stats_data.clear();
            } else {
                app.save_filename.push_str(" (must end with .csv)");
            }
        }
        KeyCode::Enter if app.input_mode => {
            if app.select_mode {
                app.confirm_selection(sys);
            } else if !app.input_buffer.is_empty() {
                let pattern = app.input_buffer.clone();
                app.add_pattern(pattern, sys);
            }
        }
        KeyCode::Up if app.select_mode => {
            if app.selected_process > 0 {
                app.selected_process -= 1;
            }
        }
        KeyCode::Down if app.select_mode => {
            if app.selected_process + 1 < app.candidate_processes.len() {
                app.selected_process += 1;
            }
        }
        KeyCode::Up if !app.input_mode && !app.save_prompt => {
            if !app.processes.is_empty() && app.selected_monitored_process > 0 {
                app.selected_monitored_process -= 1;
            }
        }
        KeyCode::Down if !app.input_mode && !app.save_prompt => {
            if !app.processes.is_empty() && app.selected_monitored_process + 1 < app.processes.len()
            {
                app.selected_monitored_process += 1;
            }
        }
        _ => {}
    }
    Ok(())
}
