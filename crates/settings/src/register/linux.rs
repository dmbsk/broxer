use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const AFTER_REGISTER: &str = "Registered. Broxer is now your default browser.";

pub fn register(router: &Path) -> io::Result<()> {
    let path = desktop_path()?;
    let dir = path.parent().expect("desktop path has a parent");
    fs::create_dir_all(dir)?;
    fs::write(&path, desktop_entry(router))?;
    let _ = Command::new("update-desktop-database").arg(dir).status();
    remember_current_default()?;
    run("xdg-settings", &["set", "default-web-browser", "broxer.desktop"])?;
    run("xdg-mime", &["default", "broxer.desktop", "x-scheme-handler/http", "x-scheme-handler/https"])
}

pub fn unregister() -> io::Result<()> {
    let previous = previous_path()?;
    if let Ok(name) = fs::read_to_string(&previous) {
        for (program, args) in restore_commands(name.trim()) {
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            run(program, &args)?;
        }
        fs::remove_file(&previous)?;
    }
    match fs::remove_file(desktop_path()?) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

fn desktop_path() -> io::Result<PathBuf> {
    dirs::data_dir()
        .map(|d| d.join("applications").join("broxer.desktop"))
        .ok_or_else(|| io::Error::other("no XDG data directory"))
}

fn run(program: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(program).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} failed ({status})")))
    }
}

fn previous_path() -> io::Result<PathBuf> {
    dirs::config_dir()
        .map(|d| d.join("broxer").join("previous-browser"))
        .ok_or_else(|| io::Error::other("no config directory"))
}

fn remember_current_default() -> io::Result<()> {
    let output = Command::new("xdg-settings").args(["get", "default-web-browser"]).output()?;
    if let Some(current) = remembered_default(&String::from_utf8_lossy(&output.stdout)) {
        let path = previous_path()?;
        fs::create_dir_all(path.parent().expect("previous path has a parent"))?;
        fs::write(path, current)?;
    }
    Ok(())
}

fn remembered_default(current: &str) -> Option<String> {
    let current = current.trim();
    (!current.is_empty() && current != "broxer.desktop").then(|| current.to_string())
}

fn restore_commands(previous: &str) -> Vec<(&'static str, Vec<String>)> {
    let args = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    vec![
        ("xdg-settings", args(&["set", "default-web-browser", previous])),
        ("xdg-mime", args(&["default", previous, "x-scheme-handler/http", "x-scheme-handler/https"])),
    ]
}

fn desktop_entry(router: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Broxer\n\
         Comment=Opens each link in the browser your rules choose\n\
         Exec=\"{}\" %u\n\
         Terminal=false\n\
         NoDisplay=true\n\
         MimeType=x-scheme-handler/http;x-scheme-handler/https;\n\
         Categories=Network;WebBrowser;\n",
        router.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_default_is_remembered_unless_it_is_broxer() {
        assert_eq!(remembered_default("firefox.desktop\n"), Some("firefox.desktop".into()));
        assert_eq!(remembered_default("broxer.desktop\n"), None);
        assert_eq!(remembered_default("  \n"), None);
    }

    #[test]
    fn unregister_restores_previous_default_for_both_tools() {
        let as_strings = |args: &[&str]| args.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        assert_eq!(
            restore_commands("firefox.desktop"),
            vec![
                ("xdg-settings", as_strings(&["set", "default-web-browser", "firefox.desktop"])),
                (
                    "xdg-mime",
                    as_strings(&["default", "firefox.desktop", "x-scheme-handler/http", "x-scheme-handler/https"])
                ),
            ]
        );
    }

    #[test]
    fn desktop_entry_handles_web_links_and_quotes_exec() {
        let entry = desktop_entry(Path::new("/opt/my apps/broxer"));
        assert!(entry.starts_with("[Desktop Entry]\n"));
        assert!(entry.contains("\nExec=\"/opt/my apps/broxer\" %u\n"));
        assert!(entry.contains("\nMimeType=x-scheme-handler/http;x-scheme-handler/https;\n"));
    }
}
