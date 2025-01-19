use clap::Parser;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Monitor CPU and memory usage of specific processes"
)]
pub struct Args {
    /// Update interval in seconds
    #[arg(short, long, default_value = "2")]
    pub interval: u64,
}
