use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "yell-guard")]
#[command(version, about = "Detects yelling from the microphone and logs shout events.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Run(RunArgs),
    Stats(StatsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    #[arg(long, default_value_t = -18.0)]
    pub threshold: f32,

    #[arg(long, default_value_t = 3500)]
    pub cooldown_ms: u64,

    #[arg(long, default_value_t = 2)]
    pub sustain_windows: usize,

    #[arg(long, default_value_t = 80)]
    pub window_ms: u64,

    #[arg(long, default_value_t = 44100)]
    pub sample_rate: u32,

    #[arg(long, default_value = default_log_path_string())]
    pub log_path: String,
}

#[derive(Args, Debug, Clone)]
pub struct StatsArgs {
    #[arg(long, default_value = default_log_path_string())]
    pub log_path: String,

    #[arg(long, help = "Window to summarize, e.g. 24h or 7d")]
    pub since: Option<String>,
}

fn default_log_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(".")));
    base.join("yell-guard").join("events.log")
}

fn default_log_path_string() -> String {
    default_log_path().to_string_lossy().to_string()
}

pub fn parse() -> Cli {
    Cli::parse()
}
