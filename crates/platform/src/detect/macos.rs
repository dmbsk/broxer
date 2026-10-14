use std::path::{Path, PathBuf};

use broxer_core::Browser;
use plist::Value;

use super::Detected;

pub fn installed_browsers() -> Vec<Detected> {
    let mut roots = vec![PathBuf::from("/Applications")];
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join("Applications"));
    }
    let mut found = Vec::new();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        for entry in entries.flatten() {
            let app = entry.path();
            if app.extension().is_none_or(|ext| ext != "app") {
                continue;
            }
            if let Some(detected) = inspect(&app) {
                found.push(detected);
            }
        }
    }
    found
}

fn inspect(app: &Path) -> Option<Detected> {
    let info = Value::from_file(app.join("Contents/Info.plist")).ok()?;
    if !handles_https(&info) {
        return None;
    }
    let name = info
        .as_dictionary()
        .and_then(|d| d.get("CFBundleName"))
        .and_then(Value::as_string)
        .map(str::to_string)
        .or_else(|| app.file_stem().map(|s| s.to_string_lossy().into_owned()))?;
    Some(Detected {
        name,
        browser: Browser { path: app.display().to_string(), args: vec![] },
    })
}

fn handles_https(info: &Value) -> bool {
    info.as_dictionary()
        .and_then(|d| d.get("CFBundleURLTypes"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| t.as_dictionary()?.get("CFBundleURLSchemes")?.as_array())
        .flatten()
        .any(|scheme| scheme.as_string() == Some("https"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(xml: &str) -> Value {
        Value::from_reader_xml(xml.as_bytes()).unwrap()
    }

    #[test]
    fn app_declaring_https_is_a_browser() {
        let value = info(r#"<?xml version="1.0"?><plist version="1.0"><dict>
            <key>CFBundleURLTypes</key><array><dict>
              <key>CFBundleURLSchemes</key><array><string>http</string><string>https</string></array>
            </dict></array></dict></plist>"#);
        assert!(handles_https(&value));
    }

    #[test]
    fn app_without_url_types_is_not_a_browser() {
        let value = info(r#"<?xml version="1.0"?><plist version="1.0"><dict>
            <key>CFBundleName</key><string>Notes</string></dict></plist>"#);
        assert!(!handles_https(&value));
    }

    #[test]
    fn safari_is_detected() {
        assert!(installed_browsers().iter().any(|d| d.browser.path.ends_with("Safari.app")));
    }
}
