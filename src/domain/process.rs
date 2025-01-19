use sysinfo::{Pid, System};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: Pid,
    pub cpu_usage: f32,
    pub memory_mb: f64,
}

impl ProcessInfo {
    pub fn new(name: String, pid: Pid, cpu_usage: f32, memory_mb: f64) -> Self {
        Self {
            name,
            pid,
            cpu_usage,
            memory_mb,
        }
    }
}

#[derive(Debug)]
pub struct ProcessManager {
    monitored_processes: Vec<(String, Pid)>,
    current_processes: Vec<ProcessInfo>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            monitored_processes: Vec::new(),
            current_processes: Vec::new(),
        }
    }

    pub fn add_process(&mut self, name: String, pid: Pid) {
        self.monitored_processes.push((name, pid));
    }

    pub fn remove_process(&mut self, index: usize) {
        if index < self.monitored_processes.len() {
            self.monitored_processes.remove(index);
        }
    }

    pub fn get_monitored_processes(&self) -> &[(String, Pid)] {
        &self.monitored_processes
    }

    pub fn get_current_processes(&self) -> &[ProcessInfo] {
        &self.current_processes
    }

    pub fn update_processes(&mut self, processes: Vec<ProcessInfo>) {
        self.current_processes = processes;
    }

    pub fn find_matching_processes(pattern: &str, sys: &System) -> Vec<(Pid, String)> {
        sys.processes()
            .iter()
            .filter(|(_, process)| {
                process
                    .name()
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            })
            .map(|(pid, process)| (*pid, process.name().to_string()))
            .collect()
    }
}
