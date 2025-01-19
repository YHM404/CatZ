pub fn calculate_cpu_percentage(old_time: f32, new_time: f32, elapsed_ms: u64) -> f32 {
    if elapsed_ms == 0 {
        return 0.0;
    }
    let cpu_delta = new_time - old_time;
    // Convert elapsed_ms to seconds and calculate percentage
    let elapsed_secs = elapsed_ms as f32 / 1000.0;
    let cpu_usage = (cpu_delta / elapsed_secs) * 100.0;
    // Round to 1 decimal place and ensure non-negative
    (cpu_usage * 10.0).round().max(0.0) / 10.0
}
