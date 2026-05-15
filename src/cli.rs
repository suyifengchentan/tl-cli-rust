use clap::Parser;

/// tl — a wget-like CLI download tool powered by TaiLerDownloader
#[derive(Parser, Debug)]
#[command(name = "tl", version, about, long_about = None)]
pub struct Args {
    /// URLs to download
    #[arg(value_name = "URL", required_unless_present = "init_config")]
    pub urls: Vec<String>,

    /// Output file path ("-" for stdout)
    #[arg(short = 'O', long, value_name = "FILE")]
    pub output: Option<String>,

    /// Save directory prefix (filename inferred from URL)
    #[arg(short = 'P', long, value_name = "DIR")]
    pub directory_prefix: Option<String>,

    /// Number of retries (0 = unlimited)
    #[arg(short = 't', long, default_value = "3", value_name = "N")]
    pub retries: usize,

    /// Initial retry delay in milliseconds
    #[arg(long, default_value = "1000", value_name = "MS")]
    pub retry_delay: u64,

    /// Max retry delay in milliseconds
    #[arg(long, default_value = "30000", value_name = "MS")]
    pub max_retry_delay: u64,

    /// Number of download threads
    #[arg(short = 's', long, default_value = "64", value_name = "N")]
    pub threads: usize,

    /// Chunk size in megabytes
    #[arg(long, default_value = "10", value_name = "MB")]
    pub chunk_size: usize,

    /// Speed limit in bytes per second (0 = unlimited)
    #[arg(long, default_value = "0", value_name = "BPS")]
    pub limit_rate: u64,

    /// Proxy URL (http://, socks5://, etc.)
    #[arg(long, value_name = "URL")]
    pub proxy: Option<String>,

    /// Add custom HTTP header (repeatable)
    #[arg(long = "header", value_name = "K:V")]
    pub headers: Vec<String>,

    /// Skip TLS certificate verification
    #[arg(long)]
    pub insecure: bool,

    /// Connection timeout in seconds
    #[arg(long, default_value = "30", value_name = "SECS")]
    pub timeout: u64,

    /// Bind to a specific local address
    #[arg(long, value_name = "ADDR")]
    pub bind_address: Option<String>,

    /// Do not resume partial downloads
    #[arg(long)]
    pub no_resume: bool,

    /// Quiet mode (suppress progress, only fatal errors)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// User-Agent header (overrides config file)
    #[arg(long, value_name = "STR")]
    pub user_agent: Option<String>,

    /// Path to config file
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<String>,

    /// Write default config to stdout and exit
    #[arg(long)]
    pub init_config: bool,
}
