use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::logger::{parse_since, read_entries};

#[derive(Debug, Clone)]
pub struct Summary {
    pub total_events: usize,
    pub events_in_window: usize,
    pub average_peak_dbfs: f64,
    pub peak_dbfs: f64,
    pub busiest_hour: Option<u32>,
}

pub fn print_summary(log_path: &str, since: Option<&str>) -> anyhow::Result<()> {
    let entries = read_entries(log_path, since)?;
    let summary = summarize(&entries);
    println!("total events: {}", summary.total_events);
    if let Some(window) = since {
        println!("events in {}: {}", window, summary.events_in_window);
    }
    println!("average peak dBFS: {:.2}", summary.average_peak_dbfs);
    println!("peak dBFS: {:.2}", summary.peak_dbfs);
    match summary.busiest_hour {
        Some(hour) => println!("busiest hour of day: {hour}:00"),
        None => println!("busiest hour of day: n/a"),
    }
    Ok(())
}

pub fn summarize(entries: &[crate::logger::LogEntry]) -> Summary {
    if entries.is_empty() {
        return Summary {
            total_events: 0,
            events_in_window: 0,
            average_peak_dbfs: 0.0,
            peak_dbfs: 0.0,
            busiest_hour: None,
        };
    }

    let total_events = entries.len();
    let sum: f64 = entries.iter().map(|e| e.peak_dbfs as f64).sum();
    let average = sum / total_events as f64;
    let peak = entries.iter().map(|e| e.peak_dbfs as f64).fold(f64::NEG_INFINITY, f64::max);

    let mut counts = HashMap::<u32, usize>::new();
    for entry in entries {
        let ts = DateTime::parse_from_rfc3339(&entry.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(Utc::now());
        let hour = ts.hour();
        *counts.entry(hour).or_insert(0) += 1;
    }

    let busiest_hour = counts.into_iter().max_by_key(|(_, count)| *count).map(|(hour, _)| hour);

    Summary {
        total_events,
        events_in_window: total_events,
        average_peak_dbfs: average,
        peak_dbfs: peak,
        busiest_hour,
    }
}
