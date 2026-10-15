use std::path::Path;

pub fn read(dir: &Path) -> String {
    imp::read(dir)
}

pub fn write(dir: &Path, record: &str) {
    imp::write(dir, record)
}

pub fn forget(dir: &Path) {
    imp::forget(dir)
}

// Defender scans a freshly written file when the next run opens it (~4 ms); registry values are not scanned.
#[cfg(windows)]
mod imp {
    use std::path::Path;

    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};

    const KEY: &str = r"Software\Broxer\LastRoute";

    pub fn read(dir: &Path) -> String {
        read_from(KEY, dir)
    }

    pub fn write(dir: &Path, record: &str) {
        write_to(KEY, dir, record)
    }

    pub fn forget(dir: &Path) {
        forget_in(KEY, dir)
    }

    pub(super) fn forget_in(key: &str, dir: &Path) {
        if let Ok(k) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(key, KEY_SET_VALUE) {
            let _ = k.delete_value(dir.as_os_str());
        }
    }

    // One value per config directory, so each config keeps its own guard as with the file.
    pub(super) fn read_from(key: &str, dir: &Path) -> String {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(key)
            .and_then(|k| k.get_value(dir.as_os_str()))
            .unwrap_or_default()
    }

    pub(super) fn write_to(key: &str, dir: &Path, record: &str) {
        if let Ok((k, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(key) {
            let _ = k.set_value(dir.as_os_str(), &record);
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::Path;

    pub fn read(dir: &Path) -> String {
        std::fs::read_to_string(dir.join("last-route")).unwrap_or_default()
    }

    pub fn write(dir: &Path, record: &str) {
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(dir.join("last-route"), record);
    }

    pub fn forget(dir: &Path) {
        let _ = std::fs::remove_file(dir.join("last-route"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn round_trips_per_directory() {
        let root = tempfile::tempdir().unwrap();
        let (a, b) = (root.path().join("a"), root.path().join("b"));
        assert_eq!(read(&a), "");
        write(&a, "https://a.dev/\n1\n1\n");
        write(&b, "https://b.dev/\n2\n1\n");
        assert_eq!(read(&a), "https://a.dev/\n1\n1\n");
        assert_eq!(read(&b), "https://b.dev/\n2\n1\n");
        forget(&a);
        assert_eq!(read(&a), "");
        assert_eq!(read(&b), "https://b.dev/\n2\n1\n");
    }

    #[cfg(windows)]
    #[test]
    fn round_trips_per_directory() {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        struct TestKey(String);
        impl Drop for TestKey {
            fn drop(&mut self) {
                let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
            }
        }

        let key = TestKey(format!(r"Software\BroxerTest-{}-last-route", std::process::id()));
        let (a, b) = (Path::new(r"C:\cfg\a"), Path::new(r"C:\cfg\b"));
        assert_eq!(imp::read_from(&key.0, a), "");
        imp::write_to(&key.0, a, "https://a.dev/\n1\n1\n");
        imp::write_to(&key.0, b, "https://b.dev/\n2\n1\n");
        assert_eq!(imp::read_from(&key.0, a), "https://a.dev/\n1\n1\n");
        assert_eq!(imp::read_from(&key.0, b), "https://b.dev/\n2\n1\n");
        imp::forget_in(&key.0, a);
        assert_eq!(imp::read_from(&key.0, a), "");
        assert_eq!(imp::read_from(&key.0, b), "https://b.dev/\n2\n1\n");
    }
}
