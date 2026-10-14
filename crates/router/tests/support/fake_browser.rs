#![cfg_attr(windows, windows_subsystem = "windows")]
// Test double: first argument is an output file, the rest are recorded one per line.
// Windowed like real browsers. Its start time (µs since epoch) goes to `<out>.started` so tests can time router start → browser launch.
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let started = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let mut args = std::env::args().skip(1);
    let out = args.next().expect("output path");
    let rest: Vec<String> = args.collect();
    std::fs::write(format!("{out}.started"), started.to_string()).unwrap();
    let tmp = format!("{out}.tmp");
    std::fs::write(&tmp, rest.join("\n")).unwrap();
    std::fs::rename(tmp, out).unwrap();
}
