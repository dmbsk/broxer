mod common;

use std::process::Command;

use common::{BROXER, Setup, received};

#[test]
fn binary_routes_url_from_argument() {
    let s = Setup::new();
    s.save(&s.config());
    let status = Command::new(BROXER)
        .arg("https://github.com/dmbsk")
        .env("BROXER_CONFIG", s.config_path())
        .env("BROXER_QUIET", "1")
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(received(&s.out("chrome")).as_deref(), Some("https://github.com/dmbsk"));
}
