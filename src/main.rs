mod alert;
mod audio;
mod config;
mod detector;
mod logger;
mod metrics;

use anyhow::{Context, Result};
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    let cli = config::Cli::parse();

    match cli.command {
        config::Command::Run(args) => run_live(args),
        config::Command::Stats(args) => metrics::print_summary(&args.log_path, args.since.as_deref()),
    }
}

fn run_live(args: config::RunArgs) -> Result<()> {
    println!("Starting Yell Guard. Listening for microphone input...");

    let mut logger = logger::Logger::new(args.log_path.clone())?;
    let mut detector = detector::Detector::new(args.threshold, args.sustain_windows, args.cooldown_ms);
    let (capture, rx) = audio::AudioCapture::new_with_config(args.sample_rate, args.window_ms)
        .with_context(|| "Could not initialize microphone capture. Ensure a microphone is available and permissions are granted.")?;
    let _capture = capture;

    let running = Arc::new(AtomicBool::new(true));
    let shutdown = Arc::clone(&running);
    ctrlc::set_handler(move || shutdown.store(false, Ordering::SeqCst))?;

    loop {
        if !running.load(Ordering::SeqCst) {
            println!("Shutdown requested. Flushing final log entries...");
            break;
        }

        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(window) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                if let Some(event) = detector.update(window.rms_dbfs, now_ms) {
                    if let Err(err) = alert::emit(&event) {
                        eprintln!("alert failed: {err}");
                    }
                    if let Err(err) = logger.append_event(&event) {
                        eprintln!("failed to append event log: {err}");
                    }
                    println!(
                        "Yell detected: {:.1} dBFS for {} ms",
                        event.peak_dbfs, event.duration_ms
                    );
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("microphone stream disconnected");
                break;
            }
        }
    }

    Ok(())
}
