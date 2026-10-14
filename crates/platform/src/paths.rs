use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    std::env::var_os("BROXER_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path)
}

pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("broxer")
        .join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn default_path_is_broxer_config_toml() {
        assert!(default_config_path().ends_with(Path::new("broxer").join("config.toml")));
    }
}
