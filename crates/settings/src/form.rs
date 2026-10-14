use std::collections::BTreeMap;

use broxer_core::{Browser, Config, Rule};
use broxer_platform::detect::Detected;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BrowserRow {
    pub name: String,
    pub path: String,
    pub args: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuleRow {
    pub domains: String,
    pub browser: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Form {
    pub browsers: Vec<BrowserRow>,
    pub rules: Vec<RuleRow>,
    pub fallback: String,
}

impl Form {
    pub fn from_config(config: &Config) -> Form {
        Form {
            browsers: config
                .browsers
                .iter()
                .map(|(name, b)| BrowserRow {
                    name: name.clone(),
                    path: b.path.clone(),
                    args: join_args(&b.args),
                })
                .collect(),
            rules: config
                .rules
                .iter()
                .map(|r| RuleRow { domains: r.domains.join(", "), browser: r.browser.clone() })
                .collect(),
            fallback: config.fallback.clone(),
        }
    }

    pub fn from_detected(found: &[Detected]) -> Form {
        let mut form = Form::default();
        for d in found {
            let base = slug(&d.name);
            let mut name = base.clone();
            let mut n = 2;
            while form.browsers.iter().any(|b| b.name == name) {
                name = format!("{base}-{n}");
                n += 1;
            }
            form.browsers.push(BrowserRow {
                name,
                path: d.browser.path.clone(),
                args: join_args(&d.browser.args),
            });
        }
        form.fallback = form.browsers.first().map(|b| b.name.clone()).unwrap_or_default();
        form
    }

    pub fn to_config(&self) -> Result<Config, String> {
        let mut browsers = BTreeMap::new();
        for row in &self.browsers {
            let name = row.name.trim();
            if name.is_empty() {
                return Err("every browser needs a name".into());
            }
            if row.path.trim().is_empty() {
                return Err(format!("browser '{name}' needs a path"));
            }
            let browser = Browser { path: row.path.trim().to_string(), args: split_args(&row.args) };
            if browsers.insert(name.to_string(), browser).is_some() {
                return Err(format!("browser name '{name}' is used twice"));
            }
        }
        let rules = self
            .rules
            .iter()
            .map(|r| Rule {
                domains: r.domains.split(',').map(str::to_string).collect(),
                browser: r.browser.clone(),
            })
            .collect();
        let mut config = Config { fallback: self.fallback.clone(), browsers, rules };
        config.normalize();
        config.validate().map_err(|e| e.to_string())?;
        Ok(config)
    }

    pub fn browser_for_test_link(&self, link: &str) -> Result<String, String> {
        let link = link.trim();
        let url = if link.contains("://") { link.to_string() } else { format!("https://{link}") };
        Ok(self.to_config()?.browser_for(&url).to_string())
    }

    pub fn move_rule(&mut self, index: usize, up: bool) {
        let other = if up { index.checked_sub(1) } else { Some(index + 1) };
        if let Some(other) = other.filter(|&o| o < self.rules.len()) {
            self.rules.swap(index, other);
        }
    }

    pub fn browser_names(&self) -> Vec<String> {
        self.browsers.iter().map(|b| b.name.trim().to_string()).collect()
    }
}

pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() { "browser".into() } else { out }
}

pub fn split_args(text: &str) -> Vec<String> {
    let (mut args, mut current) = (Vec::new(), String::new());
    let (mut quoted, mut started) = (false, false);
    for c in text.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            c if c.is_whitespace() && !quoted => {
                if started {
                    args.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            c => {
                current.push(c);
                started = true;
            }
        }
    }
    if started {
        args.push(current);
    }
    args
}

pub fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() || a.contains(char::is_whitespace) {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            fallback: "zen".into(),
            browsers: [
                ("zen".to_string(), Browser { path: "/zen".into(), args: vec![] }),
                (
                    "chrome".to_string(),
                    Browser { path: "/chrome".into(), args: vec!["--profile-directory=Profile 2".into()] },
                ),
            ]
            .into(),
            rules: vec![
                Rule { domains: vec!["youtube.com".into(), "youtu.be".into()], browser: "zen".into() },
                Rule { domains: vec!["github.com".into()], browser: "chrome".into() },
            ],
        }
    }

    fn row(name: &str, path: &str) -> BrowserRow {
        BrowserRow { name: name.into(), path: path.into(), args: String::new() }
    }

    #[test]
    fn config_round_trips_through_form() {
        let form = Form::from_config(&config());
        assert_eq!(form.rules[0].domains, "youtube.com, youtu.be");
        assert_eq!(form.to_config().unwrap(), config());
    }

    #[test]
    fn domains_are_split_and_normalized() {
        let mut form = Form::from_config(&config());
        form.rules[0].domains = "YouTube.com, https://youtu.be/ ,".into();
        assert_eq!(form.to_config().unwrap().rules[0].domains, vec!["youtube.com", "youtu.be"]);
    }

    #[test]
    fn args_with_quotes_round_trip() {
        let args = split_args(r#"--profile-directory="Profile 2"  -P work"#);
        assert_eq!(args, vec!["--profile-directory=Profile 2", "-P", "work"]);
        assert_eq!(join_args(&args), r#""--profile-directory=Profile 2" -P work"#);
        assert_eq!(split_args(&join_args(&args)), args);
        assert!(split_args("   ").is_empty());
    }

    #[test]
    fn duplicate_browser_name_is_rejected() {
        let mut form = Form::from_config(&config());
        form.browsers.push(row("zen", "/other"));
        assert_eq!(form.to_config(), Err("browser name 'zen' is used twice".into()));
    }

    #[test]
    fn browser_without_name_or_path_is_rejected() {
        let mut form = Form::from_config(&config());
        form.browsers.push(row(" ", "/x"));
        assert_eq!(form.to_config(), Err("every browser needs a name".into()));
        form.browsers.pop();
        form.browsers.push(row("edge", ""));
        assert_eq!(form.to_config(), Err("browser 'edge' needs a path".into()));
    }

    #[test]
    fn rule_with_unknown_browser_reports_rule_number() {
        let mut form = Form::from_config(&config());
        form.rules[1].browser = "edge".into();
        assert!(form.to_config().unwrap_err().contains("rule 2"));
    }

    #[test]
    fn detected_browsers_get_unique_slugs_and_first_is_fallback() {
        let detected = |name: &str| Detected {
            name: name.into(),
            browser: Browser { path: format!("/{name}"), args: vec![] },
        };
        let form = Form::from_detected(&[
            detected("Google Chrome"),
            detected("Google Chrome"),
            detected("Zen Browser"),
        ]);
        assert_eq!(form.browser_names(), vec!["google-chrome", "google-chrome-2", "zen-browser"]);
        assert_eq!(form.fallback, "google-chrome");
        assert!(form.rules.is_empty());
    }

    #[test]
    fn test_link_accepts_bare_domain() {
        let form = Form::from_config(&config());
        assert_eq!(form.browser_for_test_link("github.com/x"), Ok("chrome".into()));
        assert_eq!(form.browser_for_test_link(" https://youtu.be/x "), Ok("zen".into()));
        assert_eq!(form.browser_for_test_link("example.org"), Ok("zen".into()));
    }

    #[test]
    fn slug_handles_symbols_and_empty_names() {
        assert_eq!(slug("  Firefox (Nightly)! "), "firefox-nightly");
        assert_eq!(slug("!!!"), "browser");
    }

    #[test]
    fn move_rule_swaps_and_ignores_edges() {
        let mut form = Form::from_config(&config());
        form.move_rule(1, true);
        assert_eq!(form.rules[0].browser, "chrome");
        form.move_rule(0, true);
        assert_eq!(form.rules[0].browser, "chrome");
        form.move_rule(1, false);
        assert_eq!(form.rules[1].browser, "zen");
        form.move_rule(0, false);
        assert_eq!(form.rules[0].browser, "zen");
    }
}
