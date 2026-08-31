use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::detector::YellEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub peak_dbfs: f32,
    pub duration_ms: u64,
}

pub struct Logger {
    file: BufWriter<File>,
}

impl Logger {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file: BufWriter::new(file),
        })
    }

    pub fn append_event(&mut self, event: &YellEvent) -> Result<()> {
        let entry = LogEntry {
            timestamp: event.timestamp.to_rfc3339(),
            peak_dbfs: event.peak_dbfs,
            duration_ms: event.duration_ms,
        };
        let serialized = serde_json::to_string(&entry)?;
        writeln!(self.file, "{serialized}")?;
        self.file.flush()?;
        Ok(())
    }
}

pub fn read_entries(path: impl AsRef<Path>, since: Option<&str>) -> Result<Vec<LogEntry>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)?;
    let cutoff = match since {
        Some(value) => Some(parse_since(value)?),
        None => None,
    };

    let mut entries = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed: LogEntry = serde_json::from_str(trimmed)?;
        if let Some(cutoff) = cutoff {
            let timestamp = DateTime::parse_from_rfc3339(&parsed.timestamp)?;
            if timestamp.with_timezone(&Utc) < cutoff {
                continue;
            }
        }
        entries.push(parsed);
    }
    Ok(entries)
}

pub fn parse_since(value: &str) -> Result<DateTime<Utc>> {
    let normalized = value.trim().to_ascii_lowercase();
    let number = normalized
        .trim_end_matches(|c: char| c == 'h' || c == 'd' || c == 'w')
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("invalid duration: {value}. Use 24h, 7d, or 2w"))?;

    let duration = if normalized.ends_with('w') {
        chrono::Duration::weeks(number)
    } else if normalized.ends_with('d') {
        chrono::Duration::days(number)
    } else if normalized.ends_with('h') {
        chrono::Duration::hours(number)
    } else {
        return Err(anyhow::anyhow!("invalid duration: {value}. Use 24h, 7d, or 2w"));
    };

    Ok(Utc::now() - duration)
}
