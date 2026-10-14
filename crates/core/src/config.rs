use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::matching::{domain_matches, host_of, normalize_domain};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub fallback: String,
    #[serde(default)]
    pub browsers: BTreeMap<String, Browser>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Browser {
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(rename = "match")]
    pub domains: Vec<String>,
    pub browser: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    NotFound,
    Io(String),
    Parse(String),
    UnknownFallback(String),
    UnknownBrowser { rule: usize, browser: String },
    EmptyRule(usize),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "config file not found"),
            Self::Io(e) => write!(f, "could not read or write config: {e}"),
            Self::Parse(e) => write!(f, "invalid config: {e}"),
            Self::UnknownFallback(b) => {
                write!(f, "fallback browser '{b}' is not defined in [browsers]")
            }
            Self::UnknownBrowser { rule, browser } => write!(
                f,
                "rule {} uses browser '{browser}', which is not defined in [browsers]",
                rule + 1
            ),
            Self::EmptyRule(rule) => write!(f, "rule {} has no domains", rule + 1),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn parse(text: &str) -> Result<Config, ConfigError> {
        // Notepad can save UTF-8 with a BOM, which the TOML parser rejects.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut config: Config =
            toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.normalize();
        config.validate()?;
        Ok(config)
    }

    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(ConfigError::NotFound),
            Err(e) => Err(ConfigError::Io(e.to_string())),
        }
    }

    pub fn normalize(&mut self) {
        for rule in &mut self.rules {
            rule.domains = rule
                .domains
                .iter()
                .map(|d| normalize_domain(d))
                .filter(|d| !d.is_empty())
                .collect();
        }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.browsers.contains_key(&self.fallback) {
            return Err(ConfigError::UnknownFallback(self.fallback.clone()));
        }
        for (i, rule) in self.rules.iter().enumerate() {
            if rule.domains.is_empty() {
                return Err(ConfigError::EmptyRule(i));
            }
            if !self.browsers.contains_key(&rule.browser) {
                return Err(ConfigError::UnknownBrowser { rule: i, browser: rule.browser.clone() });
            }
        }
        Ok(())
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).expect("Config only holds strings, maps and lists")
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        self.validate()?;
        let io = |e: std::io::Error| ConfigError::Io(e.to_string());
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        // Write-then-rename so a router starting mid-save never reads a half-written file.
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, self.to_toml()).map_err(io)?;
        std::fs::rename(&tmp, path).map_err(io)
    }

    pub fn browser_for(&self, url: &str) -> &str {
        host_of(url)
            .and_then(|host| {
                self.rules
                    .iter()
                    .find(|rule| rule.domains.iter().any(|d| domain_matches(&host, d)))
            })
            .map_or(self.fallback.as_str(), |rule| rule.browser.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
fallback = "zen"

[browsers]
zen = { path = 'C:\Zen\zen.exe' }
chrome = { path = 'C:\Chrome\chrome.exe', args = ["--profile-directory=Profile 2"] }

[[rules]]
match = ["YouTube.com", "youtu.be"]
browser = "zen"

[[rules]]
match = ["https://github.com/"]
browser = "chrome"
"#;

    fn sample() -> Config {
        Config::parse(SAMPLE).unwrap()
    }

    #[test]
    fn parses_sample_and_normalizes_domains() {
        let config = sample();
        assert_eq!(config.fallback, "zen");
        assert_eq!(config.browsers["chrome"].path, r"C:\Chrome\chrome.exe");
        assert_eq!(config.browsers["chrome"].args, vec!["--profile-directory=Profile 2"]);
        assert!(config.browsers["zen"].args.is_empty());
        assert_eq!(config.rules[0].domains, vec!["youtube.com", "youtu.be"]);
        assert_eq!(config.rules[1].domains, vec!["github.com"]);
    }

    #[test]
    fn utf8_bom_is_ignored() {
        assert_eq!(Config::parse(&format!("\u{feff}{SAMPLE}")).unwrap(), sample());
    }

    #[test]
    fn unknown_fallback_is_rejected() {
        let text = SAMPLE.replacen(r#"fallback = "zen""#, r#"fallback = "safari""#, 1);
        assert_eq!(
            Config::parse(&text),
            Err(ConfigError::UnknownFallback("safari".into()))
        );
    }

    #[test]
    fn unknown_rule_browser_is_rejected() {
        let text = SAMPLE.replacen(r#"browser = "chrome""#, r#"browser = "edge""#, 1);
        assert_eq!(
            Config::parse(&text),
            Err(ConfigError::UnknownBrowser { rule: 1, browser: "edge".into() })
        );
    }

    #[test]
    fn rule_with_only_blank_domains_is_rejected() {
        let text = SAMPLE.replacen(r#"match = ["https://github.com/"]"#, r#"match = ["  "]"#, 1);
        assert_eq!(Config::parse(&text), Err(ConfigError::EmptyRule(1)));
    }

    #[test]
    fn misspelled_key_is_a_parse_error() {
        let text = SAMPLE.replacen(r#"browser = "zen""#, r#"browsr = "zen""#, 1);
        assert!(matches!(Config::parse(&text), Err(ConfigError::Parse(_))));
    }

    #[test]
    fn missing_file_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Config::load(&dir.path().join("none.toml")),
            Err(ConfigError::NotFound)
        );
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        sample().save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), sample());
    }

    #[test]
    fn save_refuses_invalid_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut config = sample();
        config.fallback = "missing".into();
        assert_eq!(
            config.save(&path),
            Err(ConfigError::UnknownFallback("missing".into()))
        );
        assert!(!path.exists());
    }

    #[test]
    fn browser_for_picks_rule_or_fallback() {
        let config = sample();
        assert_eq!(config.browser_for("https://gist.github.com/x"), "chrome");
        assert_eq!(config.browser_for("https://youtu.be/abc"), "zen");
        assert_eq!(config.browser_for("https://example.com/"), "zen");
        assert_eq!(config.browser_for("garbage"), "zen");
    }

    #[test]
    fn first_matching_rule_wins() {
        let mut config = sample();
        config.rules.insert(
            0,
            Rule { domains: vec!["gist.github.com".into()], browser: "zen".into() },
        );
        assert_eq!(config.browser_for("https://gist.github.com/x"), "zen");
        assert_eq!(config.browser_for("https://github.com/x"), "chrome");
    }
}
