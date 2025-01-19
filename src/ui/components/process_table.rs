use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::domain::process::ProcessInfo;

pub fn render_process_table(
    frame: &mut Frame,
    processes: &[ProcessInfo],
    selected_index: usize,
    area: Rect,
) {
    let header = Row::new(vec![
        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
        Cell::from("PID").style(Style::default().fg(Color::Yellow)),
        Cell::from("CPU%").style(Style::default().fg(Color::Yellow)),
        Cell::from("Memory(MB)").style(Style::default().fg(Color::Yellow)),
    ])
    .height(1);

    let table = if processes.is_empty() {
        Table::new(
            vec![Row::new(vec![
                Cell::from("No processes monitored. Press 'a' to add a process.")
                    .style(Style::default().fg(Color::Yellow)),
            ])],
            [Constraint::Percentage(100)],
        )
    } else {
        let rows = processes.iter().enumerate().map(|(i, process)| {
            let style = if i == selected_index {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(process.name.as_str()).style(style),
                Cell::from(format!("{:>5.1}", process.pid)).style(style),
                Cell::from(format!("{:>5.1}", process.cpu_usage)).style(style),
                Cell::from(format!("{:.1}", process.memory_mb)).style(style),
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
        area,
    );
}
