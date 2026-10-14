#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use broxer::{Context, route};
use broxer_core::{Browser, Config, Rule};
use broxer_platform::detect::Detected;

pub const FAKE: &str = env!("CARGO_BIN_EXE_fake-browser");
pub const BROXER: &str = env!("CARGO_BIN_EXE_broxer");

pub struct Setup {
    pub dir: tempfile::TempDir,
}

impl Setup {
    pub fn new() -> Self {
        Setup { dir: tempfile::tempdir().unwrap() }
    }

    pub fn config_path(&self) -> PathBuf {
        self.dir.path().join("config.toml")
    }

    pub fn out(&self, name: &str) -> PathBuf {
        self.dir.path().join(format!("{name}.out"))
    }

    pub fn fake(&self, name: &str) -> Browser {
        Browser { path: FAKE.into(), args: vec![self.out(name).display().to_string()] }
    }

    /// zen is the fallback; github.com goes to chrome.
    pub fn config(&self) -> Config {
        Config {
            fallback: "zen".into(),
            browsers: [("zen".to_string(), self.fake("zen")), ("chrome".to_string(), self.fake("chrome"))]
                .into(),
            rules: vec![Rule { domains: vec!["github.com".into()], browser: "chrome".into() }],
        }
    }

    pub fn save(&self, config: &Config) {
        config.save(&self.config_path()).unwrap();
    }

    pub fn route(&self, url: &str, detected: Vec<Detected>) {
        let path = self.config_path();
        route(url, &Context { config_path: &path, detect: &move || detected.clone(), popups: false });
    }

    pub fn log(&self) -> String {
        std::fs::read_to_string(self.dir.path().join("broxer.log")).unwrap_or_default()
    }
}

pub fn received(out: &Path) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(out) {
            return Some(text);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    None
}

pub fn assert_not_received(out: &Path) {
    std::thread::sleep(Duration::from_millis(300));
    assert!(!out.exists(), "{} unexpectedly received a link", out.display());
}
