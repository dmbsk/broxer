use std::path::Path;

use broxer_core::Browser;

use crate::launch::is_broxer;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
use linux as imp;
#[cfg(target_os = "macos")]
use macos as imp;
#[cfg(windows)]
use windows as imp;

#[derive(Debug, Clone, PartialEq)]
pub struct Detected {
    pub name: String,
    pub browser: Browser,
}

pub fn installed_browsers() -> Vec<Detected> {
    imp::installed_browsers()
        .into_iter()
        .filter(|d| !is_broxer(Path::new(&d.browser.path)))
        .collect()
}
