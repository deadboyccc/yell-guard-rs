use anyhow::Result;
use notify_rust::Notification;

use crate::detector::YellEvent;

pub fn emit(event: &YellEvent) -> Result<()> {
    let beep = std::thread::spawn(move || {
        let _ = play_beep();
    });

    let notif = Notification::new()
        .summary("Yell Guard")
        .body(&format!("Loud voice detected at {:.1} dBFS", event.peak_dbfs))
        .show();

    match notif {
        Ok(_) => {}
        Err(err) => {
            eprintln!("desktop notification unavailable: {err}");
        }
    }

    let _ = beep.join();
    Ok(())
}

fn play_beep() -> Result<()> {
    use std::f32::consts::PI;
    use std::io::Write;

    let sample_rate = 44100_u32;
    let duration_samples = (sample_rate as f32 * 0.15) as usize;
    let mut buf = Vec::with_capacity(duration_samples);
    for i in 0..duration_samples {
        let t = i as f32 / sample_rate as f32;
        let amplitude = (2.0 * PI * 800.0 * t).sin();
        let value = amplitude * 0.4;
        let sample = (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        buf.push(sample);
    }

    let (_stream, handle) = rodio::OutputStream::try_default()?;
    let sink = rodio::Sink::try_new(&handle)?;
    let source = rodio::buffer::SamplesBuffer::<i16>::new(1, sample_rate, buf);
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
