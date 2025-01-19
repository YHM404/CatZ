pub mod components;

use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::domain::state::{AppMode, AppState};
use components::{process_table, save_dialog};

pub fn render(frame: &mut Frame, state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status line
        ])
        .split(frame.size());

    render_title(frame, state, main_layout[0]);
    render_main_content(frame, state, main_layout[1]);
    render_status_line(frame, state, main_layout[2]);

    if state.mode == AppMode::SavePrompt {
        let area = save_dialog::centered_rect(60, 20, frame.size());
        save_dialog::render_save_dialog(frame, &state.save_filename, area);
    }
}

fn render_title(frame: &mut Frame, state: &AppState, area: Rect) {
    let title_text = match state.mode {
        AppMode::InputPattern => {
            format!("CatZ - Adding pattern: {}_", state.input_buffer)
        }
        AppMode::SelectProcess => format!(
            "CatZ - Select process to monitor ({} matches)\n(↑↓:select, Enter:confirm, q:cancel)",
            state.candidate_processes.len()
        ),
        AppMode::Stats => "CatZ - STATS MODE (s:stop stats, q:quit)".to_string(),
        _ => "CatZ\n(a:add process, s:stats, q:quit)".to_string(),
    };

    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(title, area);
}

pub fn render_main_content(frame: &mut Frame, state: &AppState, area: Rect) {
    match state.mode {
        AppMode::SelectProcess => render_process_selection(frame, state, area),
        _ => process_table::render_process_table(
            frame,
            state.processes(),
            state.selected_monitored_process,
            area,
        ),
    }
}

fn render_process_selection(frame: &mut Frame, state: &AppState, area: Rect) {
    let processes: Vec<ListItem> = state
        .candidate_processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let content = if i == state.selected_process {
                format!("> {}", p)
            } else {
                format!("  {}", p)
            };
            ListItem::new(content)
        })
        .collect();

    let process_list = List::new(processes)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Process"),
        )
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(process_list, area);
}

fn render_status_line(frame: &mut Frame, state: &AppState, area: Rect) {
    let status = match state.mode {
        AppMode::InputPattern => "Enter process name pattern (Enter to search)",
        AppMode::SelectProcess => "↑↓:select process, Enter:confirm, q:cancel",
        _ => "a:add process, d:remove selected process, s:stats mode, q:quit",
    };

    let status_widget = Paragraph::new(status).style(Style::default().fg(Color::Yellow));
    frame.render_widget(status_widget, area);
}
