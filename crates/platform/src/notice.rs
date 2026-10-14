use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn report(log_dir: &Path, message: &str, popup: bool) {
    append_log(log_dir, message);
    if popup {
        show(message);
    }
}

pub fn append_log(log_dir: &Path, message: &str) {
    let _ = std::fs::create_dir_all(log_dir);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let line = message.replace('\n', " | ");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("broxer.log"))
    {
        let _ = writeln!(file, "{secs} {line}");
    }
}

#[cfg(windows)]
fn show(message: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK, MessageBoxW};
    let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
    let (text, title) = (wide(message), wide("Broxer"));
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONWARNING);
    }
}

#[cfg(target_os = "macos")]
fn show(message: &str) {
    let quoted = message.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = std::process::Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(format!("display alert \"Broxer\" message \"{quoted}\""))
        .spawn();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn show(message: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["Broxer", message])
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_appends_to_log_in_given_dir() {
        let dir = tempfile::tempdir().unwrap();
        let log_dir = dir.path().join("broxer");
        report(&log_dir, "first problem", false);
        report(&log_dir, "second problem", false);
        let log = std::fs::read_to_string(log_dir.join("broxer.log")).unwrap();
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with(" first problem"));
        assert!(lines[1].ends_with(" second problem"));
    }
}
