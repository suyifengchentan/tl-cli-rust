# tl - Command Line Download Tool

A wget-like CLI download tool powered by [TaiLerDownloader](https://github.com/TaiLerDownloader/TaiLerDownloader), supporting multi-threaded downloads, resume, multiple protocols, and more.

## Features

- **Multi-threaded downloading** — configurable thread count and chunk size
- **Resume interrupted downloads** — `.part` files preserved on Ctrl+C, auto-resume on restart
- **Multi-protocol** — HTTP/HTTPS, FTP, BitTorrent/Magnet, ED2K
- **Speed limiting** — global rate limit in bytes per second
- **Proxy support** — HTTP, HTTPS, SOCKS5
- **Custom headers** — repeatable `--header` flag, YAML config file
- **Quiet / Verbose modes** — `-q` for scripts, `-v` for debugging
- **stdout pipe** — `-O -` streams content to stdout, progress on stderr

## Quick Install

```sh
curl -fsSL https://raw.githubusercontent.com/suyifengchentan/tl-cli-rust/main/install.sh | sh
```

Or download the binary from [Releases](https://github.com/suyifengchentan/tl-cli-rust/releases).

## Usage

```sh
# Simple download
tl https://example.com/file.zip

# Custom output file
tl -O myfile.zip https://example.com/file.zip

# Pipe to stdout
tl -O - https://example.com/data.csv | python analyze.py

# Save to directory (filename from URL)
tl -P ./downloads https://example.com/file.zip

# Multiple files (concurrent)
tl https://a.com/1.zip https://b.com/2.zip https://c.com/3.zip

# 8 threads, 20MB chunks, 5 retries
tl -s 8 --chunk-size 20 -t 5 https://example.com/large.iso

# Speed limit (1MB/s)
tl --limit-rate 1048576 https://example.com/file.zip

# SOCKS5 proxy
tl --proxy socks5://127.0.0.1:1080 https://example.com/file.zip

# Custom headers
tl --header "Referer: https://example.com" --header "Cookie: token=xxx" https://example.com/file.zip

# Skip TLS verification
tl --insecure https://self-signed.example.com/file.zip

# Force re-download (no resume)
tl --no-resume https://example.com/file.zip

# Quiet mode (no progress bar)
tl -q https://example.com/file.zip

# Generate default config
tl --init-config > ~/.config/tlcli/config.yaml
```

## Configuration

Config file search order:
1. `--config` / `-c` flag
2. `~/.config/tlcli/config.yaml`
3. `./tlcli.yaml` (current directory)

```yaml
# ~/.config/tlcli/config.yaml
http:
  user_agent: tlcli/0.1.0
  headers:
    Accept: "*/*"
    Accept-Encoding: "gzip, deflate"
  insecure: false
  timeout: 30
  bind_address: ""

download:
  threads: 64
  chunk_size_mb: 10
  max_retries: 3
  retry_delay_ms: 1000
  max_retry_delay_ms: 30000
  limit_rate: 0
  resume: true
  output_dir: ""

proxy:
  url: ""
```

CLI flags override config file values, which override built-in defaults.

## All Options

| Flag | Default | Description |
|------|---------|-------------|
| `-O`, `--output <FILE>` | — | Output file path (`-` for stdout) |
| `-P`, `--directory-prefix <DIR>` | — | Save directory (filename from URL) |
| `-t`, `--retries <N>` | 3 | Retry count (0 = unlimited) |
| `--retry-delay <MS>` | 1000 | Initial retry delay (ms) |
| `--max-retry-delay <MS>` | 30000 | Max retry delay (ms) |
| `-s`, `--threads <N>` | 64 | Download threads |
| `--chunk-size <MB>` | 10 | Chunk size (MB) |
| `--limit-rate <BPS>` | 0 | Speed limit in bytes/sec (0 = unlimited) |
| `--proxy <URL>` | — | Proxy URL (http://, socks5://) |
| `--header <K:V>` | — | Add HTTP header (repeatable) |
| `--insecure` | false | Skip TLS verification |
| `--timeout <SECS>` | 30 | Connection timeout (seconds) |
| `--bind-address <ADDR>` | — | Bind to local address |
| `--no-resume` | false | Force re-download |
| `-q`, `--quiet` | false | Suppress progress |
| `-v`, `--verbose` | false | Verbose output |
| `--user-agent <STR>` | — | User-Agent (overrides config) |
| `-c`, `--config <FILE>` | — | Config file path |
| `--init-config` | — | Print default config and exit |
| `-h`, `--help` | — | Print help |
| `-V`, `--version` | — | Print version |

## Build from Source

```sh
git clone https://github.com/suyifengchentan/tl-cli-rust.git
cd tl-downloader
cargo build --release
# Binary at: target/release/tl
```

## License

MIT License. Powered by [TaiLerDownloader](https://github.com/TaiLerDownloader/TaiLerDownloader) (AGPL v3).
