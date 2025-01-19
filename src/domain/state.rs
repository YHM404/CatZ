use crate::domain::process::{ProcessInfo, ProcessManager};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    InputPattern,
    SelectProcess,
    Stats,
    SavePrompt,
}

#[derive(Debug)]
pub struct AppState {
    pub mode: AppMode,
    pub should_quit: bool,
    pub input_buffer: String,
    pub save_filename: String,
    pub candidate_processes: Vec<String>,
    pub selected_process: usize,
    pub selected_monitored_process: usize,
    pub interval: Duration,
    pub last_tick: Instant,
    pub stats_data: Vec<(String, Vec<ProcessInfo>)>,
    process_manager: ProcessManager,
}

impl AppState {
    pub fn new(interval: u64) -> Self {
        Self {
            mode: AppMode::Normal,
            should_quit: false,
            input_buffer: String::new(),
            save_filename: String::new(),
            candidate_processes: Vec::new(),
            selected_process: 0,
            selected_monitored_process: 0,
            interval: Duration::from_secs(interval),
            last_tick: Instant::now(),
            stats_data: Vec::new(),
            process_manager: ProcessManager::new(),
        }
    }

    pub fn process_manager(&mut self) -> &mut ProcessManager {
        &mut self.process_manager
    }

    pub fn processes(&self) -> &[ProcessInfo] {
        self.process_manager.get_current_processes()
    }

    pub fn cancel_input(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.candidate_processes.clear();
    }

    pub fn tick(&mut self) {
        self.last_tick = Instant::now();
    }

    pub fn add_stats(&mut self, process_info: ProcessInfo) {
        if self.mode == AppMode::Stats {
            if let Some(entry) = self
                .stats_data
                .iter_mut()
                .find(|e| e.0 == process_info.name)
            {
                entry.1.push(process_info);
            } else {
                self.stats_data
                    .push((process_info.name.clone(), vec![process_info]));
            }
        }
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.process_manager.update_processes(processes);
    }

    pub fn clear_stats(&mut self) {
        self.stats_data.clear();
    }
}
