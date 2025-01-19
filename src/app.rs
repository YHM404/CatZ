use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    InputPattern,
    SelectProcess,
    Stats,
    SavePrompt,
}

pub struct App {
    pub processes: Vec<(String, Pid, f32, f64)>,
    pub interval: Duration,
    pub last_tick: Instant,
    pub should_quit: bool,
    pub mode: Mode,
    pub input_buffer: String,
    pub stats_data: Vec<(String, Vec<(f32, Pid, f64)>)>,
    pub save_filename: String,
    pub candidate_processes: Vec<String>,
    pub selected_process: usize,
    pub monitored_processes: Vec<(String, Pid)>,
    pub selected_monitored_process: usize,
}

impl App {
    pub fn new(interval: u64) -> Self {
        Self {
            processes: Vec::new(),
            interval: Duration::from_secs(interval),
            last_tick: Instant::now(),
            should_quit: false,
            mode: Mode::Normal,
            input_buffer: String::new(),
            stats_data: Vec::new(),
            save_filename: String::new(),
            candidate_processes: Vec::new(),
            selected_process: 0,
            monitored_processes: Vec::new(),
            selected_monitored_process: 0,
        }
    }

    pub fn add_pattern(&mut self, pattern: String, sys: &System) {
        // Find all matching processes
        let candidates: Vec<(Pid, String)> = sys
            .processes()
            .iter()
            .filter(|(_, process)| {
                process
                    .name()
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            })
            .map(|(pid, process)| (*pid, process.name().to_string()))
            .collect();

        if !candidates.is_empty() {
            self.candidate_processes = candidates.iter().map(|(_, name)| name.clone()).collect();
            self.mode = Mode::SelectProcess;
            self.selected_process = 0;
            self.input_buffer = pattern;
        }
    }

    pub fn confirm_selection(&mut self, sys: &System) {
        // Get the selected process's PID
        let selected_name = &self.candidate_processes[self.selected_process];
        if let Some((pid, _)) = sys
            .processes()
            .iter()
            .find(|(_, process)| process.name() == selected_name)
        {
            // Add to monitored processes
            self.monitored_processes.push((selected_name.clone(), *pid));
        }
        self.mode = Mode::Normal;
        self.input_buffer.clear();
        self.candidate_processes.clear();
    }

    pub fn update(&mut self, sys: &mut System, last_cpu_values: &mut HashMap<Pid, f32>) {
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(self.last_tick).as_millis() as u64;

        // Store current CPU times
        let mut current_cpu_values: HashMap<Pid, f32> = HashMap::new();
        for (pid, process) in sys.processes() {
            current_cpu_values.insert(*pid, process.cpu_usage());
        }

        sys.refresh_all();

        self.processes = sys
            .processes()
            .iter()
            .filter(|(pid, _)| self.monitored_processes.iter().any(|(_, p)| *p == **pid))
            .map(|(pid, process)| {
                let cpu_usage = if let (Some(old_time), Some(new_time)) =
                    (last_cpu_values.get(pid), current_cpu_values.get(pid))
                {
                    crate::utils::calculate_cpu_percentage(*old_time, *new_time, elapsed)
                } else {
                    process.cpu_usage()
                };
                let memory = process.memory() as f64 / 1024.0 / 1024.0;
                (process.name().to_string(), *pid, cpu_usage, memory)
            })
            .collect();

        // Update statistics if in stats mode
        if self.mode == Mode::Stats {
            for (name, pid, cpu, mem) in &self.processes {
                if let Some(entry) = self.stats_data.iter_mut().find(|e| e.0 == *name) {
                    entry.1.push((*cpu, *pid, *mem));
                } else {
                    self.stats_data
                        .push((name.clone(), vec![(*cpu, *pid, *mem)]));
                }
            }
        }

        *last_cpu_values = current_cpu_values;
        self.last_tick = current_time;
    }

    pub fn tick(&mut self) {
        self.should_quit = false;
    }

    pub fn remove_selected_process(&mut self) {
        if !self.processes.is_empty() {
            if let Some(name) = self
                .processes
                .get(self.selected_monitored_process)
                .map(|p| p.0.clone())
            {
                if let Some(pos) = self
                    .monitored_processes
                    .iter()
                    .position(|(n, _)| n == &name)
                {
                    self.monitored_processes.remove(pos);
                }
            }
            if self.selected_monitored_process >= self.processes.len().saturating_sub(1) {
                self.selected_monitored_process = self.processes.len().saturating_sub(1);
            }
        }
    }

    pub fn cancel_input(&mut self) {
        self.mode = Mode::Normal;
        self.input_buffer.clear();
        self.candidate_processes.clear();
    }
}
