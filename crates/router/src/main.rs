#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::Command;

use broxer::{Context, route};
use broxer_platform::detect::installed_browsers;
use broxer_platform::paths::config_path;

fn main() {
    // Older macOS appends a -psn_ process serial number argument.
    match std::env::args().skip(1).find(|a| !a.starts_with("-psn_")) {
        Some(url) => route_url(&url),
        None => no_url(),
    }
}

fn route_url(url: &str) {
    let config_path = config_path();
    route(
        url,
        &Context {
            config_path: &config_path,
            detect: &installed_browsers,
            popups: std::env::var_os("BROXER_QUIET").is_none(),
        },
    );
}

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
fn no_url() {
    macos::run();
}

#[cfg(not(target_os = "macos"))]
fn no_url() {
    open_settings();
}

#[cfg(target_os = "macos")]
fn open_settings() {
    let _ = Command::new("/usr/bin/open").args(["-b", "dev.dmbsk.broxer-settings"]).spawn();
}

#[cfg(not(target_os = "macos"))]
fn open_settings() {
    if let Ok(exe) = std::env::current_exe() {
        let settings = exe.with_file_name(format!("broxer-settings{}", std::env::consts::EXE_SUFFIX));
        let _ = Command::new(settings).spawn();
    }
}
