mod common;

use std::process::Command;
use std::time::{Duration, Instant};

use common::{BROXER, Setup};

#[test]
#[ignore = "timing test: cargo test --release -p broxer --test speed -- --ignored"]
fn median_routing_time_is_within_budget() {
    let s = Setup::new();
    s.save(&s.config());
    let mut times: Vec<Duration> = (0..100)
        .map(|i| {
            let start = Instant::now();
            let status = Command::new(BROXER)
                .arg(format!("https://github.com/{i}"))
                .env("BROXER_CONFIG", s.config_path())
                .env("BROXER_QUIET", "1")
                .status()
                .unwrap();
            assert!(status.success());
            start.elapsed()
        })
        .collect();
    times.sort();
    let median = times[times.len() / 2];
    println!("median {median:?}, fastest {:?}, slowest {:?}", times[0], times[99]);
    assert!(median <= Duration::from_millis(20), "median {median:?}, slowest {:?}", times[99]);
}
