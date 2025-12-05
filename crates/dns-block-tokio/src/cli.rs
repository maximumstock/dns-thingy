use std::str::FromStr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ServerArgs {
    /// DNS server to forward to
    #[arg(short, long, default_value_t = String::from("1.1.1.1:53"))]
    pub dns_relay: String, // TODO: Add support for multiple DNS servers

    /// Port to listen on
    #[arg(long, default_value_t = String::from("0.0.0.0") )]
    pub bind_address: String,

    /// Port to listen on
    #[arg(long, default_value_t = 53000)]
    pub bind_port: u16,

    /// Domains to block from being resolved
    #[arg(long, value_parser, use_value_delimiter = true)]
    pub blocked_domains: Vec<String>,

    /// Source URLs for domain lists to block from being resolved
    #[arg(long, value_parser, use_value_delimiter = true)]
    pub domain_blacklists: Vec<String>,

    /// Comma-separated tuples of `<domain>:<ip>` that describe how to resolve
    /// a domain to a static IP.
    ///
    /// The IPs can only be in IPv4 format as of now. The domains are not validated.
    #[arg(long, value_parser = clap::value_parser!(DomainRewrite))]
    pub domain_rewrites: Vec<DomainRewrite>,

    /// Whether to disable logging
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// DNS response caching is enabled by default and can be explicitly disabled
    #[arg(short, long, default_value_t = true)]
    pub caching_enabled: bool,

    /// Whether benchmark mode is enabled, ie. if forwarding should be skipped and to avoid network calls upstream
    #[arg(long, default_value_t = false)]
    pub benchmark: bool,

    /// Milliseconds of resolution delay of DNS queries when `benchmarking = true`
    #[arg(long, default_value_t = 500)]
    pub resolution_delay_ms: u64,

    /// Folder path to save DNS query recordings to
    #[arg(short, long)]
    pub recording_folder: Option<String>,
}

impl ServerArgs {
    pub fn from_env() -> Self {
        Self::parse()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DomainRewrite {
    ip: std::net::Ipv4Addr,
    domain: String,
}

impl FromStr for DomainRewrite {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains(":") {
            return Err("domain rewrite format is <domain>:<ip>".to_string());
        }

        let (domain, raw_ip) = s.split_once(":").unwrap();

        if domain.is_empty() {
            return Err("domain rewrite domain is missing".to_string());
        }

        println!("raw ip {raw_ip}");

        let ip = std::net::Ipv4Addr::from_str(raw_ip)
            .map_err(|_| format!("IP address '{raw_ip}' is not a valid IPv4 address"))?;

        Ok(DomainRewrite {
            ip,
            domain: domain.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::{net::Ipv4Addr, str::FromStr};

    use crate::cli::DomainRewrite;

    #[test]
    fn test_domain_rewrite_parse() {
        assert_eq!(
            DomainRewrite::from_str("google.com"),
            Err("domain rewrite format is <domain>:<ip>".to_string())
        );
        assert_eq!(
            DomainRewrite::from_str("google.com8.8.8.8"),
            Err("domain rewrite format is <domain>:<ip>".to_string())
        );
        assert_eq!(
            DomainRewrite::from_str("google.com:8.8.8"),
            Err("IP address '8.8.8' is not a valid IPv4 address".to_string())
        );
        assert_eq!(
            DomainRewrite::from_str("google:8.8.8.8"),
            Ok(DomainRewrite {
                ip: Ipv4Addr::from_str("8.8.8.8").unwrap(),
                domain: "google".to_string()
            })
        );
    }
}
