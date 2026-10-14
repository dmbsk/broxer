use broxer_core::Browser;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

use super::Detected;

pub fn installed_browsers() -> Vec<Detected> {
    let mut found: Vec<Detected> = Vec::new();
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(clients) = RegKey::predef(hive).open_subkey(r"SOFTWARE\Clients\StartMenuInternet")
        else {
            continue;
        };
        for key_name in clients.enum_keys().flatten() {
            let Ok(key) = clients.open_subkey(&key_name) else { continue };
            let Ok(command) = key
                .open_subkey(r"shell\open\command")
                .and_then(|c| c.get_value::<String, _>(""))
            else {
                continue;
            };
            let Some(browser) = parse_command(&command) else { continue };
            if found.iter().any(|d| d.browser.path.eq_ignore_ascii_case(&browser.path)) {
                continue;
            }
            let name = key.get_value::<String, _>("").unwrap_or(key_name);
            found.push(Detected { name, browser });
        }
    }
    found
}

fn parse_command(command: &str) -> Option<Browser> {
    let command = command.trim();
    let path = match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?,
        None => match command.to_ascii_lowercase().find(".exe") {
            Some(end) => &command[..end + 4],
            None => command.split_whitespace().next()?,
        },
    };
    (!path.is_empty()).then(|| Browser { path: path.to_string(), args: vec![] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_command_yields_exe() {
        let browser =
            parse_command(r#""C:\Program Files\Google\Chrome\Application\chrome.exe""#).unwrap();
        assert_eq!(browser.path, r"C:\Program Files\Google\Chrome\Application\chrome.exe");
        assert!(browser.args.is_empty());
    }

    #[test]
    fn quoted_command_with_trailing_args_keeps_only_exe() {
        let browser = parse_command(r#""C:\Zen\zen.exe" -osint -url "%1""#).unwrap();
        assert_eq!(browser.path, r"C:\Zen\zen.exe");
    }

    #[test]
    fn unquoted_command_yields_first_token() {
        assert_eq!(parse_command(r"C:\Opera\opera.exe --x").unwrap().path, r"C:\Opera\opera.exe");
    }

    #[test]
    fn empty_command_is_none() {
        assert_eq!(parse_command("  "), None);
    }

    #[test]
    fn unquoted_command_with_spaces_keeps_whole_exe_path() {
        let browser = parse_command(r"C:\Program Files\Internet Explorer\iexplore.exe").unwrap();
        assert_eq!(browser.path, r"C:\Program Files\Internet Explorer\iexplore.exe");
        let with_args = parse_command(r"C:\Program Files\Opera\opera.EXE --x").unwrap();
        assert_eq!(with_args.path, r"C:\Program Files\Opera\opera.EXE");
    }

    #[test]
    fn detection_runs_and_excludes_broxer() {
        assert!(crate::detect::installed_browsers().iter().all(|d| !d.name.eq_ignore_ascii_case("broxer")));
    }
}
