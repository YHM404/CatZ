use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use std::fs::File;
use std::io::Write;

use crate::domain::process::ProcessInfo;

pub fn save_stats_to_csv(
    stats: &[(String, Vec<ProcessInfo>)],
    filename: &str,
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "Process Name,PID,CPU %,Memory (MB)")?;

    for (name, data) in stats {
        for process_info in data {
            writeln!(
                file,
                "{},{},{:.2},{:.2}",
                name, process_info.pid, process_info.cpu_usage, process_info.memory_mb
            )?;
        }
    }

    Ok(())
}

pub fn render_save_dialog(frame: &mut Frame, filename: &str, area: Rect) {
    let popup = Paragraph::new(format!(
        "Save stats as: {}\n(Enter to confirm, q to cancel)",
        filename
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Save Statistics"),
    );

    let error_msg = if !filename.ends_with(".csv") && !filename.is_empty() {
        Paragraph::new("Filename must end with .csv").style(Style::default().fg(Color::Red))
    } else {
        Paragraph::new("")
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(popup, chunks[0]);
    frame.render_widget(error_msg, chunks[1]);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let min_width = 40;
    let min_height = 6;

    let popup_width = (r.width * percent_x / 100).max(min_width);
    let popup_height = (r.height * percent_y / 100).max(min_height);

    let popup_x = ((r.width - popup_width) / 2).max(0);
    let popup_y = ((r.height - popup_height) / 2).max(0);

    Rect::new(
        r.x + popup_x,
        r.y + popup_y,
        popup_width.min(r.width),
        popup_height.min(r.height),
    )
}
