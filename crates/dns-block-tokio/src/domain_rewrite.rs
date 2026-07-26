use std::{error::Error, fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DomainRewrite {
    ip: std::net::Ipv4Addr,
    domain: String,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DomainRewriteError {
    Format,
    DomainMissing,
    IpInvalid(String),
}

impl Display for DomainRewriteError {
    // Could use thiserror but this is fine
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainRewriteError::Format => {
                f.write_str("domain rewrite: required format is <domain>:<ip>")
            }
            DomainRewriteError::DomainMissing => f.write_str("domain rewrite: domain is missing"),
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

    use crate::domain_rewrite::{DomainRewrite, DomainRewriteError};

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
