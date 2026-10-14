use std::path::Path;

use broxer_core::{Config, ConfigError};
use broxer_platform::detect::Detected;
use broxer_platform::launch::launch;
use broxer_platform::notice::report;

pub struct Context<'a> {
    pub config_path: &'a Path,
    pub detect: &'a dyn Fn() -> Vec<Detected>,
    pub popups: bool,
}

pub fn route(url: &str, ctx: &Context) {
    let log_dir = ctx.config_path.parent().unwrap_or(Path::new("."));
    let problem = match Config::load(ctx.config_path) {
        Ok(config) => match launch_from_config(&config, url) {
            Ok(None) => return,
            Ok(Some(warning)) => return report(log_dir, &warning, ctx.popups),
            Err(problem) => problem,
        },
        Err(ConfigError::NotFound) => format!(
            "no config at {}; open Broxer Settings to create one",
            ctx.config_path.display()
        ),
        Err(e) => e.to_string(),
    };
    let message = match (ctx.detect)().into_iter().find(|d| launch(&d.browser, url).is_ok()) {
        Some(d) => format!("{problem}\nOpened the link in {} instead.", d.name),
        None => format!("{problem}\nNo browser could be started for {url}"),
    };
    report(log_dir, &message, ctx.popups);
}

/// Ok(None): opened as configured. Ok(Some(warning)): opened in the fallback.
fn launch_from_config(config: &Config, url: &str) -> Result<Option<String>, String> {
    let name = config.browser_for(url);
    let error = match launch(&config.browsers[name], url) {
        Ok(()) => return Ok(None),
        Err(e) => format!("could not start browser '{name}': {e}"),
    };
    if name != config.fallback && launch(&config.browsers[&config.fallback], url).is_ok() {
        return Ok(Some(format!(
            "{error}\nOpened the link in fallback browser '{}' instead.",
            config.fallback
        )));
    }
    Err(error)
}
