use std::net::IpAddr;

/// Parses the body of a domain blocklist into a list of domains.
///
/// Supported line formats:
/// - hosts entries: `0.0.0.0 example.com` (any IP, any whitespace, optional inline `#` comment)
/// - adblock domain rules: `||example.com^` (rules with modifiers or paths are skipped)
/// - bare domains: `example.com`
///
/// Comment lines (`!`, `#`) and adblock exceptions (`@@`) are ignored.
pub fn parse_blocklist(body: &str) -> Vec<String> {
    body.lines().flat_map(parse_line).collect()
}

fn parse_line(line: &str) -> Vec<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('!') || line.starts_with('#') || line.starts_with("@@") {
        return vec![];
    }

    let line = line.split('#').next().unwrap_or("").trim();

    if let Some(rule) = line.strip_prefix("||") {
        return rule
            .strip_suffix('^')
            .filter(|domain| !domain.contains(['$', '/', '^', '*']))
            .and_then(normalize_domain)
            .into_iter()
            .collect();
    }

    let mut tokens = line.split_whitespace().peekable();
    if tokens
        .peek()
        .is_some_and(|first| first.parse::<IpAddr>().is_ok())
    {
        tokens.next();
    }

    tokens.filter_map(normalize_domain).collect()
}

fn normalize_domain(token: &str) -> Option<String> {
    let domain = token.to_ascii_lowercase();
    (domain.contains('.') && domain != "localhost" && domain.parse::<IpAddr>().is_err())
        .then_some(domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adguard_hosts_list() {
        let body = "!\n! Title: The Big List\n! Version: 1.0.14.54\n!\n0.0.0.0 023pc.cn\n0.0.0.0 0q2.sitey.me\n";
        assert_eq!(parse_blocklist(body), ["023pc.cn", "0q2.sitey.me"]);
    }

    #[test]
    fn handles_crlf_tabs_and_other_ips() {
        let body = "127.0.0.1\ta.com\r\n0.0.0.0   b.com  # note\r\n:: c.com\r\n";
        assert_eq!(parse_blocklist(body), ["a.com", "b.com", "c.com"]);
    }

    #[test]
    fn skips_comments_and_junk() {
        let body = "# comment\n\n127.0.0.1 localhost\n0.0.0.0 0.0.0.0\nnodot\n";
        assert!(parse_blocklist(body).is_empty());
    }

    #[test]
    fn parses_adblock_domain_rules() {
        let body = "||ads.example.com^\n@@||ok.example.com^\n||x.com^$important\n||y.com/path^\n";
        assert_eq!(parse_blocklist(body), ["ads.example.com"]);
    }

    #[test]
    fn parses_bare_domains() {
        assert_eq!(parse_blocklist("Example.COM\n"), ["example.com"]);
    }
}
