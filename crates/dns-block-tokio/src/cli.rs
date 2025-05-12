use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct ServerArgs {
    /// DNS server to forward to
    /// TODO: Add support for multiple DNS servers
    #[arg(short, long, default_value_t = String::from("1.1.1.1:53"))]
    pub dns_relay: String,

    /// Port to listen on
    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    pub bind_address: String,

    /// Port to listen on
    #[arg(long, default_value_t = 53000)]
    pub bind_port: u16,

    /// Whether benchmark mode is enabled, ie. if forwarding should be skipped and to avoid network calls upstream
    #[arg(long, default_value_t = false)]
    pub benchmark: bool,

    /// Milliseconds of resolution delay of DNS queries when `benchmarking = true`
    #[arg(long, default_value_t = 500)]
    pub resolution_delay_ms: u64,

    /// Folder path to save DNS query recordings to
    #[arg(short, long)]
    pub recording_folder: Option<String>,

    /// Whether to disable logging
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Enables DNS reply caching
    #[arg(short, long, default_value_t = false)]
    pub caching_enabled: bool,

    /// Domains to block from being resolved
    #[arg(long, value_parser, use_value_delimiter = true)]
    pub blocked_domains: Vec<String>,

    /// Source URLs for domain lists to block from being resolved
    #[arg(long, value_parser, use_value_delimiter = true)]
    pub domain_blacklists: Vec<String>,
}

impl ServerArgs {
    pub fn from_env() -> Self {
        Self::parse()
    }
}
