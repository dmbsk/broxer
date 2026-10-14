mod common;

use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use common::{BROXER, Setup, received};

#[test]
#[ignore = "timing test: cargo test --release -p broxer --test speed -- --ignored"]
fn median_time_from_router_start_to_browser_launch_is_within_budget() {
    let s = Setup::new();
    s.save(&s.config());
    let out = s.out("chrome");
    let started = format!("{}.started", out.display());
    let mut times: Vec<Duration> = (0..100)
        .map(|i| {
            let _ = std::fs::remove_file(&out);
            let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
            let status = Command::new(BROXER)
                .arg(format!("https://github.com/{i}"))
                .env("BROXER_CONFIG", s.config_path())
                .env("BROXER_QUIET", "1")
                .status()
                .unwrap();
            assert!(status.success());
            assert!(received(&out).is_some(), "browser was not launched");
            let launched: u128 = std::fs::read_to_string(&started).unwrap().parse().unwrap();
            Duration::from_micros((launched - start) as u64)
        })
        .collect();
    times.sort();
    let median = times[times.len() / 2];
    println!("median {median:?}, fastest {:?}, slowest {:?}", times[0], times[99]);
    assert!(median <= Duration::from_millis(20), "median {median:?}, slowest {:?}", times[99]);
}
