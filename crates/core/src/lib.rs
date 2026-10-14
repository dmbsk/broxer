mod config;
mod matching;

pub use config::{Browser, Config, ConfigError, Rule};
pub use matching::{domain_matches, host_of, normalize_domain};
