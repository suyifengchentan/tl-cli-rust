use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use TaiLerDownloader::core::downloader::{DEFAULT_ED2K_GATEWAYS, DEFAULT_TORRENT_TRACKERS};

use crate::cli::Args;

/// User-facing TOML config file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub download: DownloadConfig,
    #[serde(default)]
    pub proxy: ProxyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpConfig {
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub preflight_user_agent: String,
    #[serde(default)]
    pub insecure: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadConfig {
    #[serde(default = "default_threads")]
    pub threads: usize,
    #[serde(default = "default_chunk_size")]
    pub chunk_size_mb: usize,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_ms: u64,
    #[serde(default)]
    pub limit_rate: u64,
    #[serde(default = "default_resume")]
    pub resume: bool,
    #[serde(default)]
    pub output_dir: String,
    #[serde(default = "default_ed2k_gateways")]
    pub ed2k_gateways: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub url: String,
}

/// Final merged settings used to build TLD DownloadConfig.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergedConfig {
    pub user_agent: String,
    pub preflight_user_agent: String,
    pub headers: HashMap<String, String>,
    pub insecure: bool,
    pub timeout: u64,
    pub bind_address: String,
    pub threads: usize,
    pub chunk_size_mb: usize,
    pub max_retries: usize,
    pub retry_delay_ms: u64,
    pub max_retry_delay_ms: u64,
    pub limit_rate: u64,
    pub resume: bool,
    pub output_dir: String,
    pub proxy_url: String,
    pub ed2k_gateways: Vec<String>,
    pub torrent_trackers: Vec<String>,
}

// Default values
fn default_user_agent() -> String {
    "tlcli/0.1.0".to_string()
}
fn default_timeout() -> u64 {
    30
}
fn default_threads() -> usize {
    4
}
fn default_chunk_size() -> usize {
    50
}
fn default_max_retries() -> usize {
    3
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_max_retry_delay() -> u64 {
    30000
}
fn default_resume() -> bool {
    true
}
fn default_ed2k_gateways() -> Vec<String> {
    DEFAULT_ED2K_GATEWAYS
        .iter()
        .map(|gateway| (*gateway).to_string())
        .collect()
}
fn default_torrent_trackers() -> Vec<String> {
    DEFAULT_TORRENT_TRACKERS
        .iter()
        .map(|tracker| (*tracker).to_string())
        .collect()
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            http: HttpConfig {
                headers: {
                    let mut h = HashMap::new();
                    h.insert("User-Agent".into(), default_user_agent());
                    h.insert("Accept".into(), "*/*".into());
                    h.insert("Accept-Encoding".into(), "gzip, deflate".into());
                    h
                },
                preflight_user_agent: String::new(),
                insecure: false,
                timeout: default_timeout(),
                bind_address: String::new(),
            },
            download: DownloadConfig {
                threads: default_threads(),
                chunk_size_mb: default_chunk_size(),
                max_retries: default_max_retries(),
                retry_delay_ms: default_retry_delay(),
                max_retry_delay_ms: default_max_retry_delay(),
                limit_rate: 0,
                resume: true,
                output_dir: String::new(),
                ed2k_gateways: default_ed2k_gateways(),
            },
            proxy: ProxyConfig { url: String::new() },
        }
    }
}

/// Load config from filesystem: --config flag > XDG_CONFIG_HOME > cwd > embedded default.
pub fn load_config(args: &Args) -> AppConfig {
    let mut base = AppConfig::default();

    // Priority: --config flag, then XDG dir, then cwd
    let paths: Vec<Option<PathBuf>> = if let Some(ref p) = args.config {
        vec![Some(PathBuf::from(p))]
    } else {
        vec![
            dirs::home_dir().map(|d| d.join(".config").join("tl").join("config.toml")),
            Some(PathBuf::from("tl.toml")),
        ]
    };

    for path in paths.iter().flatten() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                    base = merge_configs(base, cfg);
                }
            }
        }
    }

    base
}

fn merge_configs(base: AppConfig, override_cfg: AppConfig) -> AppConfig {
    AppConfig {
        http: HttpConfig {
            headers: {
                let mut h = base.http.headers;
                for (k, v) in override_cfg.http.headers {
                    h.insert(k, v);
                }
                h
            },
            preflight_user_agent: if override_cfg.http.preflight_user_agent.is_empty() {
                base.http.preflight_user_agent
            } else {
                override_cfg.http.preflight_user_agent
            },
            insecure: override_cfg.http.insecure || base.http.insecure,
            timeout: pick(
                override_cfg.http.timeout,
                base.http.timeout,
                default_timeout(),
            ),
            bind_address: override_cfg.http.bind_address,
        },
        download: DownloadConfig {
            threads: pick(
                override_cfg.download.threads,
                base.download.threads,
                default_threads(),
            ),
            chunk_size_mb: pick(
                override_cfg.download.chunk_size_mb,
                base.download.chunk_size_mb,
                default_chunk_size(),
            ),
            max_retries: pick(
                override_cfg.download.max_retries,
                base.download.max_retries,
                default_max_retries(),
            ),
            retry_delay_ms: pick(
                override_cfg.download.retry_delay_ms,
                base.download.retry_delay_ms,
                default_retry_delay(),
            ),
            max_retry_delay_ms: pick(
                override_cfg.download.max_retry_delay_ms,
                base.download.max_retry_delay_ms,
                default_max_retry_delay(),
            ),
            limit_rate: override_cfg.download.limit_rate,
            resume: override_cfg.download.resume,
            output_dir: override_cfg.download.output_dir,
            ed2k_gateways: if override_cfg.download.ed2k_gateways.is_empty() {
                base.download.ed2k_gateways
            } else {
                override_cfg.download.ed2k_gateways
            },
        },
        proxy: ProxyConfig {
            url: if override_cfg.proxy.url.is_empty() {
                base.proxy.url
            } else {
                override_cfg.proxy.url
            },
        },
    }
}

/// Merge CLI flags on top of config file values. CLI flags always win.
pub fn apply_cli_overrides(mut cfg: AppConfig, args: &Args) -> MergedConfig {
    if let Some(ref ua) = args.user_agent {
        cfg.http
            .headers
            .insert("User-Agent".to_string(), ua.clone());
    }
    if let Some(ref ua) = args.preflight_user_agent {
        cfg.http.preflight_user_agent = ua.clone();
    }
    if !args.headers.is_empty() {
        for h in &args.headers {
            if let Some((k, v)) = h.split_once(':') {
                cfg.http
                    .headers
                    .insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    if args.insecure {
        cfg.http.insecure = true;
    }
    if args.timeout != 30 {
        cfg.http.timeout = args.timeout;
    }
    if let Some(ref addr) = args.bind_address {
        cfg.http.bind_address = addr.clone();
    }
    if args.threads != 64 {
        cfg.download.threads = args.threads;
    }
    if args.chunk_size != 10 {
        cfg.download.chunk_size_mb = args.chunk_size;
    }
    if args.retries != 3 {
        cfg.download.max_retries = args.retries;
    }
    if args.retry_delay != 1000 {
        cfg.download.retry_delay_ms = args.retry_delay;
    }
    if args.max_retry_delay != 30000 {
        cfg.download.max_retry_delay_ms = args.max_retry_delay;
    }
    if args.limit_rate != 0 {
        cfg.download.limit_rate = args.limit_rate;
    }
    if args.no_resume {
        cfg.download.resume = false;
    }
    if let Some(ref dir) = args.directory_prefix {
        cfg.download.output_dir = dir.clone();
    }
    if let Some(ref proxy) = args.proxy {
        cfg.proxy.url = proxy.clone();
    }

    let user_agent = resolve_user_agent(&cfg.http);

    MergedConfig {
        user_agent: user_agent.clone(),
        preflight_user_agent: resolve_preflight_user_agent(&cfg.http, &user_agent),
        headers: cfg.http.headers,
        insecure: cfg.http.insecure,
        timeout: cfg.http.timeout,
        bind_address: cfg.http.bind_address,
        threads: cfg.download.threads,
        chunk_size_mb: cfg.download.chunk_size_mb,
        max_retries: cfg.download.max_retries,
        retry_delay_ms: cfg.download.retry_delay_ms,
        max_retry_delay_ms: cfg.download.max_retry_delay_ms,
        limit_rate: cfg.download.limit_rate,
        resume: cfg.download.resume,
        output_dir: cfg.download.output_dir,
        proxy_url: cfg.proxy.url,
        ed2k_gateways: cfg.download.ed2k_gateways,
        torrent_trackers: load_tracker_list(args),
    }
}

fn pick<T: Eq>(val: T, default: T, builtin: T) -> T {
    if val == builtin {
        default
    } else {
        val
    }
}

fn resolve_user_agent(cfg: &HttpConfig) -> String {
    cfg.headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("user-agent"))
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(default_user_agent)
}

fn resolve_preflight_user_agent(cfg: &HttpConfig, user_agent: &str) -> String {
    if cfg.preflight_user_agent.is_empty() {
        user_agent.to_string()
    } else {
        cfg.preflight_user_agent.clone()
    }
}

/// Generate the default TOML config content.
pub fn default_config_toml() -> String {
    let cfg = AppConfig::default();
    let mut out = toml::to_string_pretty(&cfg).unwrap_or_default();
    out.insert_str(
        0,
        "# tl config\n# Location: ~/.config/tl/config.toml\n# BitTorrent trackers: tracker_list.txt in the same directory\n\n",
    );
    out
}

pub fn primary_config_path(args: &Args) -> PathBuf {
    if let Some(ref path) = args.config {
        return PathBuf::from(path);
    }

    let cwd_path = PathBuf::from("tl.toml");
    if cwd_path.exists() {
        return cwd_path;
    }

    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("tl")
        .join("config.toml")
}

pub fn tracker_list_path(args: &Args) -> PathBuf {
    let config_path = primary_config_path(args);
    let config_dir = config_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("tracker_list.txt")
}

pub fn default_tracker_list_txt() -> String {
    default_torrent_trackers().join("\n")
}

pub fn load_tracker_list(args: &Args) -> Vec<String> {
    let tracker_path = tracker_list_path(args);
    if let Ok(content) = std::fs::read_to_string(tracker_path) {
        parse_line_list(&content)
    } else {
        let defaults = default_torrent_trackers();
        if let Some(parent) = tracker_list_path(args).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(tracker_list_path(args), default_tracker_list_txt());
        defaults
    }
}

fn parse_line_list(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToString::to_string)
        .collect()
}
