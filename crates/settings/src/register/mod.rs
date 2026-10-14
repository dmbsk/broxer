use std::io;
use std::path::PathBuf;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::{AFTER_REGISTER, register, unregister};
#[cfg(target_os = "macos")]
pub use macos::{AFTER_REGISTER, register, unregister};
#[cfg(windows)]
pub use windows::{AFTER_REGISTER, open_default_apps, register, unregister};

#[cfg(not(target_os = "macos"))]
pub fn router_path() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    existing(exe.with_file_name(format!("broxer{}", std::env::consts::EXE_SUFFIX)))
}

#[cfg(target_os = "macos")]
pub fn router_path() -> io::Result<PathBuf> {
    // .../Broxer Settings.app/Contents/MacOS/broxer-settings → .../Broxer.app
    let exe = std::env::current_exe()?;
    let dir = exe.ancestors().nth(4).ok_or_else(|| io::Error::other("unexpected app layout"))?;
    existing(dir.join("Broxer.app"))
}

fn existing(path: PathBuf) -> io::Result<PathBuf> {
    if path.exists() {
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found; keep Broxer next to Broxer Settings", path.display()),
        ))
    }
}
