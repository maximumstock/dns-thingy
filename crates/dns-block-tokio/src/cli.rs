use std::net::SocketAddr;

use clap::Parser;

use crate::domain_rewrite::DomainRewrite;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub(crate) struct ServerArgs {
    /// DNS server to forward to, as `<ip>:<port>`, e.g. `1.1.1.1:53` or `[2606:4700:4700::1111]:53`.
    /// The port is required; an IP address without a port is rejected. Hostnames are not supported.
    #[arg(short, long, default_value = "1.1.1.1:53")]
    pub dns_relay: SocketAddr, // TODO: Add support for multiple DNS servers

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

    /// A `<domain>:<ip>` tuple that describes how to resolve a domain to a static IPv4 address.
    /// Can be passed multiple times, e.g. `--rewrite-a a.com:1.2.3.4 --rewrite-a b.com:5.6.7.8`.
    ///
    /// The domain may be a wildcard like `*.a.com:1.2.3.4`, which matches subdomains at any depth
    /// (`x.a.com`, `x.y.a.com`) but not `a.com` itself. Exact rules win over wildcards, and more
    /// specific wildcards win over less specific ones.
    ///
    /// Rewritten replies use a TTL of 60 seconds.
    /// The domains are not validated.
    /// Only `A` records can be rewritten as of now.
    #[arg(long, value_parser = clap::value_parser!(DomainRewrite))]
    pub rewrite_a: Vec<DomainRewrite>,

    /// Whether to disable logging
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// DNS response caching is enabled by default, pass this flag to disable it
    #[arg(long)]
    pub no_caching: bool,

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

    pub fn caching_enabled(&self) -> bool {
        !self.no_caching
    }
}

#[cfg(test)]
mod test {
    use std::{net::SocketAddr, str::FromStr};

    use clap::{CommandFactory, Parser, error::ErrorKind};

    use crate::{cli::ServerArgs, domain_rewrite::DomainRewrite};

    fn parse(args: &[&str]) -> Result<ServerArgs, clap::Error> {
        ServerArgs::try_parse_from(std::iter::once("dns-block-tokio").chain(args.iter().copied()))
    }

    #[test]
    fn test_cli_definition() {
        ServerArgs::command().debug_assert();
    }

    #[test]
    fn test_defaults() {
        let args = parse(&[]).unwrap();
        assert_eq!(args.dns_relay, SocketAddr::from_str("1.1.1.1:53").unwrap());
        assert_eq!(args.bind_address, "0.0.0.0");
        assert_eq!(args.bind_port, 53000);
        assert!(args.blocked_domains.is_empty());
        assert!(args.domain_blacklists.is_empty());
        assert!(args.rewrite_a.is_empty());
        assert!(!args.quiet);
        assert!(args.caching_enabled());
        assert!(!args.benchmark);
        assert_eq!(args.resolution_delay_ms, 500);
        assert_eq!(args.recording_folder, None);
    }

    #[test]
    fn test_dns_relay_accepts_ip_with_port() {
        let args = parse(&["--dns-relay", "8.8.8.8:5353"]).unwrap();
        assert_eq!(
            args.dns_relay,
            SocketAddr::from_str("8.8.8.8:5353").unwrap()
        );

        let args = parse(&["-d", "[::1]:53"]).unwrap();
        assert_eq!(args.dns_relay, SocketAddr::from_str("[::1]:53").unwrap());
    }

    #[test]
    fn test_dns_relay_rejects_missing_or_invalid_port() {
        for bad in [
            "1.1.1.1",
            "::1",
            "dns.google:53",
            "1.1.1.1:",
            "1.1.1.1:99999",
        ] {
            let err = parse(&["--dns-relay", bad]).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::ValueValidation, "{bad}");
        }
    }

    #[test]
    fn test_bind_port_validation() {
        for bad in ["99999", "abc"] {
            assert!(parse(&["--bind-port", bad]).is_err(), "{bad}");
        }
        assert_eq!(parse(&["--bind-port", "5353"]).unwrap().bind_port, 5353);
    }

    #[test]
    fn test_list_args() {
        let args = parse(&[
            "--blocked-domains",
            "a.com,b.com",
            "--domain-blacklists",
            "http://x/1,http://x/2",
        ])
        .unwrap();
        assert_eq!(args.blocked_domains, ["a.com", "b.com"]);
        assert_eq!(args.domain_blacklists, ["http://x/1", "http://x/2"]);
    }

    #[test]
    fn test_rewrite_a() {
        let args = parse(&[
            "--rewrite-a",
            "a.com:1.2.3.4",
            "--rewrite-a",
            "*.b.com:5.6.7.8",
        ])
        .unwrap();
        assert_eq!(
            args.rewrite_a,
            [
                DomainRewrite::from_str("a.com:1.2.3.4").unwrap(),
                DomainRewrite::from_str("*.b.com:5.6.7.8").unwrap(),
            ]
        );
        assert!(parse(&["--rewrite-a", "a.com"]).is_err());
    }

    #[test]
    fn test_no_caching() {
        assert!(parse(&[]).unwrap().caching_enabled());
        assert!(!parse(&["--no-caching"]).unwrap().caching_enabled());
        assert!(parse(&["--caching-enabled"]).is_err());
    }

    #[test]
    fn test_optional_and_flags() {
        let args = parse(&[
            "-r",
            "/tmp/rec",
            "-q",
            "--benchmark",
            "--resolution-delay-ms",
            "10",
        ])
        .unwrap();
        assert_eq!(args.recording_folder.as_deref(), Some("/tmp/rec"));
        assert!(args.quiet);
        assert!(args.benchmark);
        assert_eq!(args.resolution_delay_ms, 10);
    }
}
