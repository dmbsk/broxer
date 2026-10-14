use std::io;
use std::path::Path;
use std::process::Command;

use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_SET_VALUE};

pub const AFTER_REGISTER: &str =
    "Registered. Now pick Broxer as your web browser in Default apps.";
const CLIENT_KEY: &str = r"Clients\StartMenuInternet\Broxer";
const PROG_ID: &str = "BroxerURL";

pub fn register(router: &Path) -> io::Result<()> {
    write_entries(&software()?, router)
}

pub fn unregister() -> io::Result<()> {
    remove_entries(&software()?)
}

pub fn open_default_apps() -> io::Result<()> {
    Command::new("explorer").arg("ms-settings:defaultapps").spawn().map(drop)
}

fn software() -> io::Result<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Software", KEY_ALL_ACCESS)
}

fn write_entries(software: &RegKey, router: &Path) -> io::Result<()> {
    let exe = router.display().to_string();
    let icon = format!("{exe},0");

    let (prog, _) = software.create_subkey(format!(r"Classes\{PROG_ID}"))?;
    prog.set_value("", &"Broxer URL")?;
    prog.create_subkey("DefaultIcon")?.0.set_value("", &icon)?;
    prog.create_subkey(r"shell\open\command")?.0.set_value("", &format!("\"{exe}\" \"%1\""))?;

    let (client, _) = software.create_subkey(CLIENT_KEY)?;
    client.set_value("", &"Broxer")?;
    client.create_subkey("DefaultIcon")?.0.set_value("", &icon)?;
    client.create_subkey(r"shell\open\command")?.0.set_value("", &format!("\"{exe}\""))?;
    let (caps, _) = client.create_subkey("Capabilities")?;
    caps.set_value("ApplicationName", &"Broxer")?;
    caps.set_value("ApplicationDescription", &"Opens each link in the browser your rules choose")?;
    caps.set_value("ApplicationIcon", &icon)?;
    let (urls, _) = caps.create_subkey("URLAssociations")?;
    urls.set_value("http", &PROG_ID)?;
    urls.set_value("https", &PROG_ID)?;

    software
        .create_subkey("RegisteredApplications")?
        .0
        .set_value("Broxer", &format!(r"Software\{CLIENT_KEY}\Capabilities"))
}

fn remove_entries(software: &RegKey) -> io::Result<()> {
    for key in [format!(r"Classes\{PROG_ID}"), CLIENT_KEY.to_string()] {
        match software.delete_subkey_all(&key) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    if let Ok(apps) = software.open_subkey_with_flags("RegisteredApplications", KEY_SET_VALUE) {
        let _ = apps.delete_value("Broxer");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRoot(String);

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    #[test]
    fn writes_and_removes_entries_under_given_root() {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let root_name = format!(r"Software\BroxerTest-{}", std::process::id());
        let (root, _) = hkcu.create_subkey(&root_name).unwrap();
        let _cleanup = TestRoot(root_name);

        write_entries(&root, Path::new(r"C:\Apps\broxer\broxer.exe")).unwrap();
        let command: String = root
            .open_subkey(r"Classes\BroxerURL\shell\open\command")
            .unwrap()
            .get_value("")
            .unwrap();
        assert_eq!(command, r#""C:\Apps\broxer\broxer.exe" "%1""#);
        let https: String = root
            .open_subkey(r"Clients\StartMenuInternet\Broxer\Capabilities\URLAssociations")
            .unwrap()
            .get_value("https")
            .unwrap();
        assert_eq!(https, "BroxerURL");
        let registered: String =
            root.open_subkey("RegisteredApplications").unwrap().get_value("Broxer").unwrap();
        assert_eq!(registered, r"Software\Clients\StartMenuInternet\Broxer\Capabilities");

        remove_entries(&root).unwrap();
        assert!(root.open_subkey(r"Classes\BroxerURL").is_err());
        assert!(root.open_subkey(CLIENT_KEY).is_err());
        assert!(
            root.open_subkey("RegisteredApplications")
                .unwrap()
                .get_value::<String, _>("Broxer")
                .is_err()
        );
        remove_entries(&root).unwrap();
    }
}
