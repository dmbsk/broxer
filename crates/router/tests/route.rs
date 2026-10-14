mod common;

use broxer_platform::detect::Detected;
use common::{BROXER, Setup, assert_not_received, received};

#[test]
fn matching_link_opens_in_rule_browser() {
    let s = Setup::new();
    s.save(&s.config());
    s.route("https://gist.github.com/x", vec![]);
    assert_eq!(received(&s.out("chrome")).as_deref(), Some("https://gist.github.com/x"));
    assert_not_received(&s.out("zen"));
    assert_eq!(s.log(), "");
}

#[test]
fn unmatched_link_opens_in_fallback() {
    let s = Setup::new();
    s.save(&s.config());
    s.route("https://www.youtube.com/watch?v=1", vec![]);
    assert_eq!(received(&s.out("zen")).as_deref(), Some("https://www.youtube.com/watch?v=1"));
}

#[test]
fn browser_args_come_before_url() {
    let s = Setup::new();
    let mut config = s.config();
    config.browsers.get_mut("chrome").unwrap().args.push("--profile-directory=Profile 2".into());
    s.save(&config);
    s.route("https://github.com/", vec![]);
    assert_eq!(
        received(&s.out("chrome")).as_deref(),
        Some("--profile-directory=Profile 2\nhttps://github.com/")
    );
}

#[test]
fn special_characters_in_url_arrive_intact() {
    let s = Setup::new();
    s.save(&s.config());
    let url = r##"https://github.com/search?q=a%20b&type=code&x="y z"#frag"##;
    s.route(url, vec![]);
    assert_eq!(received(&s.out("chrome")).as_deref(), Some(url));
}

#[test]
fn missing_browser_falls_back_and_logs() {
    let s = Setup::new();
    let mut config = s.config();
    config.browsers.get_mut("chrome").unwrap().path =
        s.dir.path().join("missing-chrome.exe").display().to_string();
    s.save(&config);
    s.route("https://github.com/", vec![]);
    assert_eq!(received(&s.out("zen")).as_deref(), Some("https://github.com/"));
    assert!(s.log().contains("could not start browser 'chrome'"), "{}", s.log());
}

#[test]
fn browser_pointing_at_broxer_is_never_launched() {
    let s = Setup::new();
    let mut config = s.config();
    config.browsers.get_mut("chrome").unwrap().path = BROXER.into();
    s.save(&config);
    s.route("https://github.com/", vec![]);
    assert_eq!(received(&s.out("zen")).as_deref(), Some("https://github.com/"));
    assert!(s.log().contains("Broxer itself"), "{}", s.log());
}

#[test]
fn missing_config_uses_detected_browser() {
    let s = Setup::new();
    let detected = vec![Detected { name: "Fake Browser".into(), browser: s.fake("detected") }];
    s.route("https://github.com/", detected);
    assert_eq!(received(&s.out("detected")).as_deref(), Some("https://github.com/"));
    let log = s.log();
    assert!(log.contains("no config at"), "{log}");
    assert!(log.contains("Opened the link in Fake Browser instead."), "{log}");
}

#[test]
fn invalid_config_uses_detected_browser() {
    let s = Setup::new();
    std::fs::write(s.config_path(), "fallback = ").unwrap();
    let detected = vec![Detected { name: "Fake Browser".into(), browser: s.fake("detected") }];
    s.route("https://github.com/", detected);
    assert_eq!(received(&s.out("detected")).as_deref(), Some("https://github.com/"));
    assert!(s.log().contains("invalid config"), "{}", s.log());
}

#[test]
fn nothing_to_launch_is_logged() {
    let s = Setup::new();
    s.route("https://github.com/", vec![]);
    assert!(
        s.log().contains("No browser could be started for https://github.com/"),
        "{}",
        s.log()
    );
}
