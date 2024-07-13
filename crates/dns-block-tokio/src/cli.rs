use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerArgs {
    /// DNS server to forward to
    /// TODO: Add support for multiple DNS servers
    #[arg(short, long, default_value_t = String::from("1.1.1.1:53"))]
    pub dns_relay: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 53000)]
    pub port: u16,

    /// Whether benchmark mode is enabled, ie. if forwarding should be skipped and to avoid network calls upstream
    #[arg(short, long, default_value_t = false)]
    pub benchmark: bool,

    /// Folder path to save DNS query recordings to
    #[arg(short, long)]
    pub recording_folder: Option<String>,

    /// Whether to disable logging
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}

impl ServerArgs {
    pub fn from_env() -> Self {
        Self::parse()
    }
}
