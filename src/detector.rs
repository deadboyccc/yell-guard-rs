use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct YellEvent {
    pub timestamp: DateTime<Utc>,
    pub peak_dbfs: f32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Detector {
    threshold_dbfs: f32,
    sustain_windows: usize,
    cooldown_ms: u64,
    consecutive_above: usize,
    active_since_ms: Option<u64>,
    active_peak_dbfs: f32,
    cooldown_until_ms: Option<u64>,
}

impl Detector {
    pub fn new(threshold_dbfs: f32, sustain_windows: usize, cooldown_ms: u64) -> Self {
        Self {
            threshold_dbfs,
            sustain_windows,
            cooldown_ms,
            consecutive_above: 0,
            active_since_ms: None,
            active_peak_dbfs: f32::NEG_INFINITY,
            cooldown_until_ms: None,
        }
    }

    pub fn update(&mut self, rms_dbfs: f32, now_ms: u64) -> Option<YellEvent> {
        if let Some(cooldown_until) = self.cooldown_until_ms {
            if now_ms < cooldown_until {
                self.consecutive_above = 0;
                self.active_since_ms = None;
                self.active_peak_dbfs = f32::NEG_INFINITY;
                return None;
            }
            self.cooldown_until_ms = None;
        }

        if rms_dbfs >= self.threshold_dbfs {
            self.consecutive_above += 1;
            if self.active_since_ms.is_none() {
                self.active_since_ms = Some(now_ms);
            }
            self.active_peak_dbfs = self.active_peak_dbfs.max(rms_dbfs);
        } else {
            self.consecutive_above = 0;
            self.active_since_ms = None;
            self.active_peak_dbfs = f32::NEG_INFINITY;
            return None;
        }

        if self.consecutive_above < self.sustain_windows {
            return None;
        }

        let start_ms = self.active_since_ms.unwrap_or(now_ms);
        let event = YellEvent {
            timestamp: Utc::now(),
            peak_dbfs: self.active_peak_dbfs,
            duration_ms: now_ms.saturating_sub(start_ms),
        };

        self.consecutive_above = 0;
        self.active_since_ms = None;
        self.active_peak_dbfs = f32::NEG_INFINITY;
        self.cooldown_until_ms = Some(now_ms + self.cooldown_ms);
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::Detector;

    #[test]
    fn sustained_loudness_triggers_event() {
        let mut detector = Detector::new(-18.0, 2, 5000);

        let event1 = detector.update(-16.0, 1000);
        assert!(event1.is_none());

        let event2 = detector.update(-15.0, 2000);
        assert!(event2.is_some());
        assert_eq!(event2.unwrap().duration_ms, 1000);
    }

    #[test]
    fn quiet_windows_reset_the_counter() {
        let mut detector = Detector::new(-18.0, 2, 5000);

        assert!(detector.update(-10.0, 1000).is_none());
        assert!(detector.update(-30.0, 2000).is_none());
        assert!(detector.update(-12.0, 3000).is_none());
        assert!(detector.update(-14.0, 4000).is_none());
    }

    #[test]
    fn cooldown_blocks_repeated_events() {
        let mut detector = Detector::new(-18.0, 1, 5000);

        let first = detector.update(-12.0, 1000);
        assert!(first.is_some());

        let second = detector.update(-10.0, 2000);
        assert!(second.is_none());
    }

    #[test]
    fn peak_dbfs_tracks_the_loudest_window() {
        let mut detector = Detector::new(-18.0, 2, 5000);

        detector.update(-16.0, 1000);
        let event = detector.update(-7.0, 2000);
        let event = event.expect("event should fire");
        assert!(event.peak_dbfs > -10.0);
    }
}
