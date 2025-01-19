use crate::domain::{
    process::ProcessManager,
    state::{AppMode, AppState},
};
use color_eyre::Result;
use crossterm::event::KeyCode;
use sysinfo::System;

pub fn handle_key_events(key_event: KeyCode, state: &mut AppState, sys: &System) -> Result<()> {
    match (key_event, state.mode) {
        (KeyCode::Char('q') | KeyCode::Esc, AppMode::SavePrompt) => {
            state.mode = AppMode::Stats;
            state.save_filename.clear();
        }
        (KeyCode::Char('q') | KeyCode::Esc, AppMode::InputPattern | AppMode::SelectProcess) => {
            state.cancel_input();
        }
        (KeyCode::Char('q') | KeyCode::Esc, _) => {
            state.should_quit = true;
        }
        (KeyCode::Char('a'), AppMode::Normal) => {
            state.mode = AppMode::InputPattern;
            state.input_buffer.clear();
        }
        (KeyCode::Char('s'), AppMode::Normal) => {
            state.mode = AppMode::Stats;
            state.clear_stats();
        }
        (KeyCode::Char('s'), AppMode::Stats) => {
            state.mode = AppMode::SavePrompt;
            state.save_filename.clear();
        }
        (KeyCode::Char('d'), AppMode::Normal) => {
            let selected = state.selected_monitored_process;
            state.process_manager().remove_process(selected);
        }
        (KeyCode::Char(c), AppMode::SavePrompt) => {
            state.save_filename.push(c);
        }
        (KeyCode::Char(c), AppMode::InputPattern) => {
            state.input_buffer.push(c);
        }
        (KeyCode::Backspace, AppMode::SavePrompt) => {
            state.save_filename.pop();
        }
        (KeyCode::Backspace, AppMode::InputPattern) => {
            state.input_buffer.pop();
        }
        (KeyCode::Enter, AppMode::SavePrompt) => {
            if state.save_filename.ends_with(".csv") {
                if !state.save_filename.is_empty() {
                    let _ = crate::ui::components::save_dialog::save_stats_to_csv(
                        &state.stats_data,
                        &state.save_filename,
                    );
                }
                state.mode = AppMode::Normal;
                state.save_filename.clear();
                state.clear_stats();
            } else {
                state.save_filename.push_str(" (must end with .csv)");
            }
        }
        (KeyCode::Enter, AppMode::SelectProcess) => {
            if let Some(selected_process) = state
                .candidate_processes
                .get(state.selected_process)
                .cloned()
            {
                let pattern = state.input_buffer.clone();
                if let Some((pid, _)) = ProcessManager::find_matching_processes(&pattern, sys)
                    .into_iter()
                    .find(|(_, name)| name == &selected_process)
                {
                    state.process_manager().add_process(selected_process, pid);
                }
            }
            state.mode = AppMode::Normal;
            state.input_buffer.clear();
            state.candidate_processes.clear();
        }
        (KeyCode::Enter, AppMode::InputPattern) => {
            if !state.input_buffer.is_empty() {
                let candidates = ProcessManager::find_matching_processes(&state.input_buffer, sys);
                state.candidate_processes = candidates.into_iter().map(|(_, name)| name).collect();
                if !state.candidate_processes.is_empty() {
                    state.mode = AppMode::SelectProcess;
                    state.selected_process = 0;
                }
            }
        }
        (KeyCode::Up, AppMode::SelectProcess) => {
            if state.selected_process > 0 {
                state.selected_process -= 1;
            }
        }
        (KeyCode::Up, AppMode::Normal) => {
            if !state.processes().is_empty() && state.selected_monitored_process > 0 {
                state.selected_monitored_process -= 1;
            }
        }
        (KeyCode::Down, AppMode::SelectProcess) => {
            if state.selected_process + 1 < state.candidate_processes.len() {
                state.selected_process += 1;
            }
        }
        (KeyCode::Down, AppMode::Normal) => {
            if !state.processes().is_empty()
                && state.selected_monitored_process + 1 < state.processes().len()
            {
                state.selected_monitored_process += 1;
            }
        }
        _ => {}
    }
    Ok(())
}
