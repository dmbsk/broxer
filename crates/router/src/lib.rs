use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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
    if is_looping(log_dir, url) {
        let message = format!(
            "{url} came back to Broxer {LOOP_REPEATS} times in a row; a configured browser probably hands links to the default browser (Broxer). Stopped to avoid a loop."
        );
        return report(log_dir, &message, ctx.popups);
    }
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

const LOOP_REPEATS: u32 = 3;
const LOOP_WINDOW_MS: u128 = 2000;

/// Records this URL; true when it is the LOOP_REPEATS-th arrival within LOOP_WINDOW_MS.
fn is_looping(log_dir: &Path, url: &str) -> bool {
    let marker = log_dir.join("last-route");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let previous = std::fs::read_to_string(&marker).unwrap_or_default();
    let mut lines = previous.lines();
    let (last_url, first_ms, count) = (
        lines.next().unwrap_or(""),
        lines.next().and_then(|l| l.parse::<u128>().ok()).unwrap_or(0),
        lines.next().and_then(|l| l.parse::<u32>().ok()).unwrap_or(0),
    );
    let (first_ms, count) = if last_url == url && now.saturating_sub(first_ms) < LOOP_WINDOW_MS {
        (first_ms, count + 1)
    } else {
        (now, 1)
    };
    let _ = std::fs::create_dir_all(log_dir);
    let _ = std::fs::write(&marker, format!("{url}\n{first_ms}\n{count}\n"));
    count >= LOOP_REPEATS
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
