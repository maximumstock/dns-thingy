use std::{collections::HashMap, error::Error, fmt::Display, net::Ipv4Addr, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DomainRewrite {
    pub(crate) ip: std::net::Ipv4Addr,
    pub(crate) domain: String,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DomainRewriteError {
    Format,
    DomainMissing,
    WildcardInvalid(String),
    IpInvalid(String),
}

/// Lookup table for domain rewrites, supporting exact and `*.suffix` wildcard rules.
///
/// Wildcards match subdomains at any depth but not the apex domain itself.
/// Exact rules win over wildcards, and longer wildcard suffixes win over shorter ones.
#[derive(Debug, Default)]
pub(crate) struct DomainRewrites {
    exact: HashMap<String, Ipv4Addr>,
    /// Keyed by the suffix without the leading `*.`, e.g. `domain.com`
    wildcard: HashMap<String, Ipv4Addr>,
}

impl FromIterator<DomainRewrite> for DomainRewrites {
    fn from_iter<T: IntoIterator<Item = DomainRewrite>>(iter: T) -> Self {
        let mut rewrites = Self::default();
        for rewrite in iter {
            match rewrite.domain.strip_prefix("*.") {
                Some(suffix) => rewrites.wildcard.insert(suffix.to_string(), rewrite.ip),
                None => rewrites.exact.insert(rewrite.domain, rewrite.ip),
            };
        }
        rewrites
    }
}

impl DomainRewrites {
    pub(crate) fn resolve(&self, domain: &str) -> Option<Ipv4Addr> {
        if let Some(ip) = self.exact.get(domain) {
            return Some(*ip);
        }

        // Left-to-right yields the longest (most specific) suffix first
        domain
            .match_indices('.')
            .find_map(|(i, _)| self.wildcard.get(&domain[i + 1..]).copied())
    }
}

impl Display for DomainRewriteError {
    // Could use thiserror but this is fine
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainRewriteError::Format => {
                f.write_str("domain rewrite: required format is <domain>:<ip>")
            }
            DomainRewriteError::DomainMissing => f.write_str("domain rewrite: domain is missing"),
            DomainRewriteError::WildcardInvalid(domain) => f.write_fmt(format_args!(
                "domain rewrite: invalid wildcard '{domain}', only a leading '*.' label is supported"
            )),
            DomainRewriteError::IpInvalid(ip) => f.write_fmt(format_args!(
                "domain rewrite: given IP address '{ip}' is not a valid IPv4 address"
            )),
        }
    }
}

impl Error for DomainRewriteError {}

impl FromStr for DomainRewrite {
    type Err = DomainRewriteError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (domain, raw_ip) = s.split_once(":").ok_or(DomainRewriteError::Format)?;

        if domain.is_empty() {
            return Err(DomainRewriteError::DomainMissing);
        }

        if domain.contains('*') {
            let valid = domain
                .strip_prefix("*.")
                .is_some_and(|suffix| !suffix.is_empty() && !suffix.contains('*'));
            if !valid {
                return Err(DomainRewriteError::WildcardInvalid(domain.into()));
            }
        }

        let ip = std::net::Ipv4Addr::from_str(raw_ip)
            .map_err(|_| DomainRewriteError::IpInvalid(raw_ip.into()))?;

        Ok(DomainRewrite {
            ip,
            domain: domain.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::{net::Ipv4Addr, str::FromStr};

    use crate::domain_rewrite::{DomainRewrite, DomainRewriteError, DomainRewrites};

    fn rewrites(rules: &[&str]) -> DomainRewrites {
        rules
            .iter()
            .map(|r| DomainRewrite::from_str(r).unwrap())
            .collect()
    }

    #[test]
    fn test_wildcard_parse() {
        assert_eq!(
            DomainRewrite::from_str("*.domain.com:1.2.3.4"),
            Ok(DomainRewrite {
                ip: Ipv4Addr::new(1, 2, 3, 4),
                domain: "*.domain.com".to_string()
            })
        );
        for bad in ["*", "*.", "a*.com", "a.*.com", "*.*.com"] {
            assert_eq!(
                DomainRewrite::from_str(&format!("{bad}:1.2.3.4")),
                Err(DomainRewriteError::WildcardInvalid(bad.into())),
                "{bad}"
            );
        }
    }

    #[test]
    fn test_wildcard_resolve() {
        let r = rewrites(&[
            "*.domain.com:1.1.1.1",
            "*.b.domain.com:2.2.2.2",
            "exact.domain.com:3.3.3.3",
        ]);
        let ip = |a| Some(Ipv4Addr::new(a, a, a, a));

        assert_eq!(r.resolve("a.domain.com"), ip(1));
        assert_eq!(r.resolve("x.y.domain.com"), ip(1));
        assert_eq!(r.resolve("a.b.domain.com"), ip(2));
        assert_eq!(r.resolve("exact.domain.com"), ip(3));
        assert_eq!(r.resolve("domain.com"), None);
        assert_eq!(r.resolve("b.domain.com"), ip(1));
        assert_eq!(r.resolve("xdomain.com"), None);
        assert_eq!(r.resolve("other.com"), None);
    }

    #[test]
    fn test_domain_rewrite_parse() {
        assert_eq!(
            DomainRewrite::from_str("google.com"),
            Err(DomainRewriteError::Format)
        );
        assert_eq!(
            DomainRewrite::from_str("google.com8.8.8.8"),
            Err(DomainRewriteError::Format)
        );
        assert_eq!(
            DomainRewrite::from_str("google.com:8.8.8"),
            Err(DomainRewriteError::IpInvalid("8.8.8".into()))
        );
        assert_eq!(
            DomainRewrite::from_str("google.com:8.8.8.8.8"),
            Err(DomainRewriteError::IpInvalid("8.8.8.8.8".into()))
        );

        assert_eq!(
            DomainRewrite::from_str("google:8.8.8.8"),
            Ok(DomainRewrite {
                ip: Ipv4Addr::from_str("8.8.8.8").unwrap(),
                domain: "google".to_string()
            })
        );
        assert_eq!(
            DomainRewrite::from_str("google.com:8.8.8.8"),
            Ok(DomainRewrite {
                ip: Ipv4Addr::from_str("8.8.8.8").unwrap(),
                domain: "google.com".to_string()
            })
        );
    }
}
