use clap::Parser;

use crate::domain_rewrite::DomainRewrite;

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
