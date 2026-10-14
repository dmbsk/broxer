use std::collections::HashSet;
use std::path::PathBuf;

use broxer_core::Browser;

use super::Detected;

pub fn installed_browsers() -> Vec<Detected> {
    let home = dirs::home_dir().unwrap_or_default();
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    let dirs = std::iter::once(data_home).chain(data_dirs.split(':').map(PathBuf::from));

    let mut seen_files = HashSet::new();
    let mut found = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir.join("applications")) else { continue };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            if !file_name.to_string_lossy().ends_with(".desktop") || !seen_files.insert(file_name) {
                continue;
            }
            if let Some(detected) = std::fs::read_to_string(entry.path())
                .ok()
                .and_then(|text| parse_desktop_entry(&text))
            {
                found.push(detected);
            }
        }
    }
    found
}

fn parse_desktop_entry(text: &str) -> Option<Detected> {
    let (mut in_main, mut hidden) = (false, false);
    let (mut name, mut exec, mut mime) = (None, None, String::new());
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            in_main = line == "[Desktop Entry]";
            continue;
        }
        let Some((key, value)) = line.split_once('=').filter(|_| in_main) else { continue };
        match key.trim() {
            "Name" => name = Some(value.trim().to_string()),
            "Exec" => exec = Some(value.trim().to_string()),
            "MimeType" => mime = value.to_string(),
            "NoDisplay" | "Hidden" => hidden |= value.trim() == "true",
            _ => {}
        }
    }
    if hidden || !mime.split(';').any(|m| m.trim() == "x-scheme-handler/https") {
        return None;
    }
    // Field codes (%u) and flatpak's @@ markers are placeholders, not arguments.
    let mut tokens: Vec<String> = exec?
        .split_whitespace()
        .filter(|t| !t.starts_with('%') && !t.starts_with("@@"))
        .map(|t| t.trim_matches('"').to_string())
        .collect();
    if tokens.is_empty() {
        return None;
    }
    let path = tokens.remove(0);
    Some(Detected { name: name?, browser: Browser { path, args: tokens } })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIREFOX: &str = "[Desktop Entry]\nName=Firefox\nExec=firefox %u\nMimeType=text/html;x-scheme-handler/http;x-scheme-handler/https;\n\n[Desktop Action new-window]\nName=New Window\nExec=firefox --new-window %u\n";

    #[test]
    fn browser_entry_is_detected() {
        let found = parse_desktop_entry(FIREFOX).unwrap();
        assert_eq!(found.name, "Firefox");
        assert_eq!(found.browser.path, "firefox");
        assert!(found.browser.args.is_empty());
    }

    #[test]
    fn flatpak_exec_keeps_arguments() {
        let text = "[Desktop Entry]\nName=Zen\nExec=/usr/bin/flatpak run app.zen_browser.zen @@u %u @@\nMimeType=x-scheme-handler/https;\n";
        let found = parse_desktop_entry(text).unwrap();
        assert_eq!(found.browser.path, "/usr/bin/flatpak");
        assert_eq!(found.browser.args, vec!["run", "app.zen_browser.zen"]);
    }

    #[test]
    fn non_browser_is_ignored() {
        let text = "[Desktop Entry]\nName=Files\nExec=nautilus %U\nMimeType=inode/directory;\n";
        assert_eq!(parse_desktop_entry(text), None);
    }

    #[test]
    fn hidden_entry_is_ignored() {
        let text = FIREFOX.replacen("Name=Firefox\n", "Name=Firefox\nNoDisplay=true\n", 1);
        assert_eq!(parse_desktop_entry(&text), None);
    }
}
