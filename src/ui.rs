use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
    },
};
use std::fs::File;
use std::io::Write;
use sysinfo::Pid;

use crate::app::App;

pub fn save_stats_to_csv(
    stats: &[(String, Vec<(f32, Pid, f64)>)],
    filename: &str,
) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "Process Name,PID,CPU %,Memory (MB)")?;

    for (name, data) in stats {
        for (cpu, pid, mem) in data {
            writeln!(file, "{},{},{},{:.2}", name, pid, cpu, mem)?;
        }
    }

    Ok(())
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Ensure minimum dimensions for the popup
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

pub fn ui(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status line
        ])
        .split(frame.size());

    // Title
    let title_text = if app.input_mode {
        if app.select_mode {
            format!(
                "Process Monitor - Select process to monitor ({} matches)\n(↑↓:select, Enter:confirm, q:cancel)",
                app.candidate_processes.len()
            )
        } else {
            format!("Process Monitor - Adding pattern: {}_", app.input_buffer)
        }
    } else if app.stats_mode {
        "Process Monitor - STATS MODE (s:stop stats, q:quit)".to_string()
    } else {
        "Process Monitor\n(a:add process, s:stats, q:quit)".to_string()
    };
    let title = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(title, main_layout[0]);

    // Process selection list or process table
    if app.select_mode {
        let processes: Vec<ListItem> = app
            .candidate_processes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let content = if i == app.selected_process {
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

        frame.render_stateful_widget(
            process_list,
            main_layout[1],
            &mut ListState::default().with_selected(Some(app.selected_process)),
        );
    } else {
        // Process table
        let header = Row::new(vec![
            Cell::from("Name").style(Style::default().fg(Color::Yellow)),
            Cell::from("PID").style(Style::default().fg(Color::Yellow)),
            Cell::from("CPU%").style(Style::default().fg(Color::Yellow)),
            Cell::from("Memory(MB)").style(Style::default().fg(Color::Yellow)),
        ])
        .height(1);

        let table = if app.processes.is_empty() {
            Table::new(
                vec![Row::new(vec![
                    Cell::from("No processes monitored. Press 'a' to add a process.")
                        .style(Style::default().fg(Color::Yellow)),
                ])],
                [Constraint::Percentage(100)],
            )
        } else {
            let rows = app
                .processes
                .iter()
                .enumerate()
                .map(|(i, (name, pid, cpu, mem))| {
                    let style = if i == app.selected_monitored_process {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    Row::new(vec![
                        Cell::from(name.as_str()).style(style),
                        Cell::from(format!("{:>5.1}", pid)).style(style),
                        Cell::from(format!("{:>5.1}", cpu)).style(style),
                        Cell::from(format!("{:.1}", mem)).style(style),
                    ])
                    .height(1)
                });

            Table::new(rows.collect::<Vec<_>>(), [
                Constraint::Percentage(40), // Name
                Constraint::Percentage(20), // PID
                Constraint::Percentage(20), // CPU%
                Constraint::Percentage(20), // Memory(MB)
            ])
            .header(header)
        };

        frame.render_widget(
            table.block(Block::default().borders(Borders::ALL).title("Processes")),
            main_layout[1],
        );
    }

    // Status line
    let status = if app.input_mode && !app.select_mode {
        Paragraph::new("Enter process name pattern (Enter to search)")
            .style(Style::default().fg(Color::Yellow))
    } else if app.select_mode {
        Paragraph::new("↑↓:select process, Enter:confirm, q:cancel")
            .style(Style::default().fg(Color::Yellow))
    } else {
        Paragraph::new("a:add process, d:remove selected process, s:stats mode, q:quit")
            .style(Style::default().fg(Color::Yellow))
    };
    frame.render_widget(status, main_layout[2]);
}

pub fn draw_save_prompt(f: &mut Frame, app: &App) {
    let popup = Paragraph::new(format!(
        "Save stats as: {}\n(Enter to confirm, q to cancel)",
        app.save_filename
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Save Statistics"),
    );

    let error_msg = if !app.save_filename.ends_with(".csv") && !app.save_filename.is_empty() {
        Paragraph::new("Filename must end with .csv").style(Style::default().fg(Color::Red))
    } else {
        Paragraph::new("")
    };

    let area = centered_rect(60, 20, f.size());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(1)])
        .split(area);

    f.render_widget(Clear, area);
    f.render_widget(popup, chunks[0]);
    f.render_widget(error_msg, chunks[1]);
}
