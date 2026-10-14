use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use broxer_core::Browser;

pub fn launch(browser: &Browser, url: &str) -> io::Result<()> {
    let path = Path::new(&browser.path);
    if is_broxer(path) {
        return Err(io::Error::other("refusing to open a link with Broxer itself"));
    }
    // Relative names (e.g. "firefox" from a Linux .desktop file) are resolved via PATH by spawn.
    if path.is_absolute() && !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not exist", browser.path),
        ));
    }
    command(path, &browser.args, url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
}

pub fn is_broxer(path: &Path) -> bool {
    path.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("broxer"))
        || path
            .components()
            .any(|c| c.as_os_str().eq_ignore_ascii_case("Broxer.app"))
}

#[cfg(target_os = "macos")]
fn command(path: &Path, args: &[String], url: &str) -> Command {
    if path.extension().is_some_and(|ext| ext == "app") {
        let mut command = Command::new("/usr/bin/open");
        command.arg("-a").arg(path).arg(url);
        if !args.is_empty() {
            command.arg("--args").args(args);
        }
        return command;
    }
    direct(path, args, url)
}

#[cfg(not(target_os = "macos"))]
fn command(path: &Path, args: &[String], url: &str) -> Command {
    direct(path, args, url)
}

fn direct(path: &Path, args: &[String], url: &str) -> Command {
    let mut command = Command::new(path);
    command.args(args).arg(url);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_broxer_executables() {
        assert!(is_broxer(Path::new("/opt/broxer/broxer")));
        assert!(is_broxer(Path::new("C:/Programs/broxer/Broxer.exe")));
        assert!(is_broxer(Path::new("/Applications/Broxer.app")));
        assert!(is_broxer(Path::new("/Applications/Broxer.app/Contents/MacOS/broxer")));
        assert!(!is_broxer(Path::new("C:/Programs/broxer/broxer-settings.exe")));
        assert!(!is_broxer(Path::new("/usr/bin/firefox")));
    }

    #[test]
    fn refuses_to_launch_broxer_itself() {
        let browser = Browser { path: "/opt/broxer/broxer".into(), args: vec![] };
        let err = launch(&browser, "https://example.com").unwrap_err();
        assert!(err.to_string().contains("Broxer itself"), "{err}");
    }

    #[test]
    fn missing_absolute_path_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let browser = Browser {
            path: dir.path().join("no-such-browser.exe").display().to_string(),
            args: vec![],
        };
        let err = launch(&browser, "https://example.com").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
