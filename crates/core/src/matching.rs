use url::Url;

pub fn normalize_domain(raw: &str) -> String {
    let mut domain = raw.trim();
    if let Some(i) = domain.find("://") {
        domain = &domain[i + 3..];
    }
    let domain = domain.trim_start_matches("*.").trim_start_matches('.');
    // Parsing as a URL lowercases, strips port/path and converts IDN to punycode like host_of does.
    match Url::parse(&format!("http://{domain}")) {
        Ok(url) => url.host_str().unwrap_or("").trim_end_matches('.').to_string(),
        Err(_) => String::new(),
    }
}

pub fn host_of(url: &str) -> Option<String> {
    let url = Url::parse(url.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    Some(url.host_str()?.trim_end_matches('.').to_string())
}

pub fn domain_matches(host: &str, domain: &str) -> bool {
    host == domain
        || (host.len() > domain.len()
            && host.ends_with(domain)
            && host.as_bytes()[host.len() - domain.len() - 1] == b'.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_host_matches() {
        assert!(domain_matches("github.com", "github.com"));
    }

    #[test]
    fn subdomain_matches() {
        assert!(domain_matches("gist.github.com", "github.com"));
    }

    #[test]
    fn lookalikes_do_not_match() {
        assert!(!domain_matches("notgithub.com", "github.com"));
        assert!(!domain_matches("github.com.evil.com", "github.com"));
        assert!(!domain_matches("com", "github.com"));
    }

    #[test]
    fn host_is_lowercased_without_port() {
        assert_eq!(host_of("https://GitHub.COM:8443/x"), Some("github.com".into()));
    }

    #[test]
    fn trailing_dot_is_trimmed_from_host() {
        assert_eq!(host_of("https://github.com./x"), Some("github.com".into()));
    }

    #[test]
    fn userinfo_is_not_the_host() {
        assert_eq!(host_of("https://github.com@evil.com/"), Some("evil.com".into()));
    }

    #[test]
    fn non_web_urls_have_no_host() {
        assert_eq!(host_of("mailto:a@github.com"), None);
        assert_eq!(host_of("file:///C:/x.html"), None);
        assert_eq!(host_of("not a url"), None);
    }

    #[test]
    fn rule_domains_are_normalized() {
        for raw in [
            "github.com",
            "GitHub.com",
            "  github.com ",
            "https://github.com/",
            "http://github.com/some/path",
            "*.github.com",
            ".github.com",
            "github.com:443",
            "github.com.",
        ] {
            assert_eq!(normalize_domain(raw), "github.com", "input: {raw:?}");
        }
    }

    #[test]
    fn idn_domains_become_punycode_on_both_sides() {
        assert_eq!(normalize_domain("Bücher.de"), "xn--bcher-kva.de");
        assert_eq!(host_of("https://BÜCHER.de/"), Some("xn--bcher-kva.de".into()));
    }

    #[test]
    fn blank_domain_normalizes_to_empty() {
        assert_eq!(normalize_domain("   "), "");
    }
}
