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
    run("xdg-settings", &["set", "default-web-browser", "broxer.desktop"])?;
    run("xdg-mime", &["default", "broxer.desktop", "x-scheme-handler/http", "x-scheme-handler/https"])
}

pub fn unregister() -> io::Result<()> {
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
    fn desktop_entry_handles_web_links_and_quotes_exec() {
        let entry = desktop_entry(Path::new("/opt/my apps/broxer"));
        assert!(entry.starts_with("[Desktop Entry]\n"));
        assert!(entry.contains("\nExec=\"/opt/my apps/broxer\" %u\n"));
        assert!(entry.contains("\nMimeType=x-scheme-handler/http;x-scheme-handler/https;\n"));
    }
}
