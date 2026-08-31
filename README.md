# Yell Guard

Yell Guard is a native Rust CLI that listens to the microphone, detects sustained loudness above a configurable threshold, and logs a JSONL event when it notices a yell.

## Features

- Continuous microphone capture with `cpal`
- Real-time RMS-to-dBFS loudness estimation
- Debounce and cooldown logic to avoid repeated triggers from one shout
- Short beep playback and desktop notification on a detected yell
- Append-only JSONL event logging with timestamps
- Summary reporting from the log file without the microphone running

## Build

```bash
cargo build
```

## Run live monitoring

```bash
cargo run -- run
```

Common options:

```bash
cargo run -- run --threshold -18 --cooldown-ms 4000 --sustain-windows 2 --log-path ~/.local/share/yell-guard/events.log
```

## Check summary stats

```bash
cargo run -- stats --log-path ~/.local/share/yell-guard/events.log
cargo run -- stats --log-path ~/.local/share/yell-guard/events.log --since 24h
```

## Threshold meaning

The threshold is measured as dBFS (decibels relative to full scale). A value around `-18` dBFS is a reasonable starting point for a sustained loud voice in a quiet room. Lower values (for example `-12`) are more sensitive; higher values (for example `-24`) are less sensitive.

The detector does not trigger on a single transient. It requires the signal to remain above threshold for multiple 50–100 ms windows, and then enforces a cooldown before another event can be raised.

## Log file behavior

Every yell creates one JSON object per line in the log file. The file is opened with append mode only, so it never truncates or overwrites previous events across restarts.

Example entry:

```json
{"timestamp":"2026-08-31T14:32:07Z","peak_dbfs":-6.2,"duration_ms":420}
```

## Notes

- No mic audio is written to disk; only event metadata is logged.
- If no microphone is available, the app exits with a clear error instead of panicking.
- On Linux, notifications use `notify-rust` via D-Bus. On macOS/Windows the same API is used; the OS may show a native notification or may require a different host integration depending on the environment.
