use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, LOCATION, SET_COOKIE, USER_AGENT};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use TaiLerDownloader::core::downloader::{
    DownloadConfig, DownloadTask, EventType, HSDownloader, UA,
};

use crate::callback::{self, CallbackEvent};
use crate::config::MergedConfig;

pub async fn run(
    urls: Vec<String>,
    cfg: MergedConfig,
    output: Option<String>,
    quiet: bool,
) -> Result<(), String> {
    let is_stdout = output.as_deref() == Some("-");

    // Build download tasks
    let tasks = build_tasks(&urls, &cfg, output.as_deref(), &cfg.output_dir, cfg.resume).await?;

    if tasks.is_empty() {
        return Err("no URLs to download".into());
    }

    let challenge_tasks: Vec<DownloadTask> = tasks
        .iter()
        .filter(|task| {
            task.headers
                .keys()
                .any(|key| key.eq_ignore_ascii_case("cookie"))
        })
        .cloned()
        .collect();

    if challenge_tasks.len() == tasks.len() {
        return download_cookie_challenge_tasks(challenge_tasks, &cfg, quiet).await;
    }

    // Show summary
    if !quiet {
        eprintln!("Downloading {} file(s):", tasks.len());
        for t in &tasks {
            eprintln!("  → {}  ({})", t.show_name, t.url);
        }
        eprintln!();
    }

    let task_count = tasks.len();

    // Set up callback channel
    let (tx, mut rx) = mpsc::unbounded_channel::<CallbackEvent>();
    callback::register_sender(tx);

    // Build TLD config
    let headers = cfg.headers.clone();
    if !cfg.user_agent.is_empty() {
        // user_agent is set directly on DownloadConfig, headers are separate
    }

    let tld_config = DownloadConfig {
        tasks,
        thread_count: cfg.threads,
        chunk_size_mb: cfg.chunk_size_mb,
        callback_func: Some(callback::get_callback()),
        use_callback_url: false,
        callback_url: None,
        use_socket: None,
        show_name: String::new(),
        user_agent: if cfg.user_agent.is_empty() {
            UA.to_string()
        } else {
            cfg.user_agent.clone()
        },
        max_retries: if cfg.max_retries == 0 {
            usize::MAX
        } else {
            cfg.max_retries
        },
        retry_delay_ms: cfg.retry_delay_ms,
        max_retry_delay_ms: cfg.max_retry_delay_ms,
        speed_limit_bps: cfg.limit_rate,
        proxy_url: if cfg.proxy_url.is_empty() {
            None
        } else {
            Some(cfg.proxy_url.clone())
        },
        headers,
    };

    let downloader = HSDownloader::new(tld_config);

    // Progress display
    let mp = if !quiet && !is_stdout {
        Some(Arc::new(MultiProgress::new()))
    } else {
        None
    };

    let pb_style = ProgressStyle::with_template(
        "{msg}\n{wide_bar} {percent:>3}%  {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}",
    )
    .unwrap();

    let pb = if let Some(ref mp) = mp {
        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(pb_style);
        if task_count > 1 {
            pb.set_message(format!("[1/{}] preparing...", task_count));
        }
        Some(pb)
    } else {
        None
    };

    // Track which tasks are active
    let active_files: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    // Spawn download
    let download_handle = tokio::spawn(async move { downloader.start_download().await });

    // Spawn progress reader
    let pb_clone = pb.clone();
    let mp_clone = mp.clone();
    let active_clone = active_files.clone();
    let progress_task = tokio::spawn(async move {
        while let Some(evt) = rx.recv().await {
            match evt.event.event_type {
                EventType::StartOne => {
                    let name = evt.event.show_name.clone();
                    let mut files = active_clone.write().await;
                    if !files.contains(&name) {
                        files.push(name.clone());
                    }
                    if let Some(ref pb) = pb_clone {
                        let msg = if files.len() > 1 {
                            format!("[{}] {}", files.len(), files.join(", "))
                        } else {
                            name
                        };
                        pb.set_message(msg);
                    }
                }
                EventType::EndOne => {
                    let name = &evt.event.show_name;
                    let mut files = active_clone.write().await;
                    files.retain(|f| f != name);
                    if let Some(ref pb) = pb_clone {
                        if files.is_empty() {
                            pb.set_message("finishing...");
                        } else {
                            pb.set_message(format!("[{}] {}", files.len(), files.join(", ")));
                        }
                    }
                }
                EventType::Update => {
                    if let Some(ref pb) = pb_clone {
                        if let Some(total) = evt.data.get("Total").and_then(|v| v.as_i64()) {
                            pb.set_length(total as u64);
                        }
                        if let Some(downloaded) =
                            evt.data.get("total_bytes").and_then(|v| v.as_i64())
                        {
                            pb.set_position(downloaded as u64);
                        }
                    }
                }
                EventType::Msg => {
                    if let Some(text) = evt.data.get("Text").and_then(|v| v.as_str()) {
                        eprintln!("  {}", text);
                    }
                }
                EventType::Err => {
                    if let Some(err) = evt.data.get("Error").and_then(|v| v.as_str()) {
                        eprintln!("  Error: {}", err);
                    }
                }
                EventType::End => {
                    if let Some(ref pb) = pb_clone {
                        pb.finish_with_message("done");
                    }
                    if let Some(ref mp) = mp_clone {
                        mp.clear().ok();
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle Ctrl+C
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let result = tokio::select! {
        res = download_handle => {
            match res {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(format!("download failed: {}", e)),
                Err(e) => Err(format!("download task panicked: {}", e)),
            }
        }
        _ = &mut ctrl_c => {
            eprintln!("\nInterrupted. Partial files preserved for resume.");
            Err("interrupted".into())
        }
    };

    // Clean up
    callback::clear_sender();
    if let Some(ref mp) = mp {
        mp.clear().ok();
    }
    let _ = progress_task.await;

    // Rename .part files on success
    if result.is_ok() {
        rename_part_files(&urls, &cfg.output_dir, output.as_deref()).await;
    }

    result
}

async fn download_cookie_challenge_tasks(
    tasks: Vec<DownloadTask>,
    cfg: &MergedConfig,
    quiet: bool,
) -> Result<(), String> {
    let client = build_http_client(cfg)?;

    for task in &tasks {
        if !quiet {
            eprintln!("Downloading {}", task.show_name);
        }
        download_cookie_challenge_task(&client, task, cfg, quiet).await?;
    }

    Ok(())
}

fn build_http_client(cfg: &MergedConfig) -> Result<reqwest::Client, String> {
    build_http_client_with_redirect_policy(cfg, reqwest::redirect::Policy::default())
}

fn build_http_preflight_client(cfg: &MergedConfig) -> Result<reqwest::Client, String> {
    build_http_client_with_redirect_policy(cfg, reqwest::redirect::Policy::none())
}

fn build_http_client_with_redirect_policy(
    cfg: &MergedConfig,
    redirect_policy: reqwest::redirect::Policy,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .redirect(redirect_policy)
        .timeout(Duration::from_secs(cfg.timeout));
    if cfg.insecure {
        builder = builder.danger_accept_invalid_certs(true);
    }
    if !cfg.bind_address.is_empty() {
        let addr: IpAddr = cfg
            .bind_address
            .parse()
            .map_err(|e| format!("invalid bind address {}: {}", cfg.bind_address, e))?;
        builder = builder.local_address(addr);
    }
    if !cfg.proxy_url.is_empty() {
        let proxy = reqwest::Proxy::all(&cfg.proxy_url)
            .map_err(|e| format!("invalid proxy URL {}: {}", cfg.proxy_url, e))?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|e| format!("failed to build HTTP client: {}", e))
}

async fn download_cookie_challenge_task(
    client: &reqwest::Client,
    task: &DownloadTask,
    cfg: &MergedConfig,
    quiet: bool,
) -> Result<(), String> {
    if let Some(parent) = Path::new(&task.save_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {}", parent.display(), e))?;
        }
    }

    let existing_len = if cfg.resume && Path::new(&task.save_path).exists() {
        std::fs::metadata(&task.save_path)
            .map(|meta| meta.len())
            .unwrap_or(0)
    } else {
        0
    };

    let mut response = build_cookie_challenge_request(client, task, cfg, existing_len)
        .send()
        .await
        .map_err(|e| format!("request failed for {}: {}", task.url, e))?;

    if existing_len > 0 && response.status() == reqwest::StatusCode::OK {
        let _ = std::fs::remove_file(&task.save_path);
        response = build_cookie_challenge_request(client, task, cfg, 0)
            .send()
            .await
            .map_err(|e| format!("request failed for {}: {}", task.url, e))?;
    }

    if !response.status().is_success() {
        return Err(format!(
            "download failed for {}: bad status {}",
            task.url,
            response.status()
        ));
    }

    let total_size = response.content_length().map(|len| len + existing_len);
    let writer = open_output_writer(&task.save_path, existing_len > 0).await?;
    let pb = if quiet {
        None
    } else {
        let pb = ProgressBar::new(total_size.unwrap_or(0));
        pb.set_style(
            ProgressStyle::with_template(
                "{msg}\n{wide_bar} {bytes}/{total_bytes}  {bytes_per_sec}  ETA {eta}",
            )
            .unwrap(),
        );
        pb.set_message(task.show_name.clone());
        pb.set_position(existing_len);
        Some(pb)
    };

    write_response_body(response, writer, pb.as_ref()).await?;

    if let Some(pb) = pb {
        pb.finish_with_message(format!("done: {}", task.show_name));
    }

    if task.save_path.ends_with(".part") {
        let final_path = task.save_path.trim_end_matches(".part");
        std::fs::rename(&task.save_path, final_path).map_err(|e| {
            format!(
                "failed to rename {} -> {}: {}",
                task.save_path, final_path, e
            )
        })?;
    }

    Ok(())
}

fn build_cookie_challenge_request(
    client: &reqwest::Client,
    task: &DownloadTask,
    cfg: &MergedConfig,
    existing_len: u64,
) -> reqwest::RequestBuilder {
    let mut request = client.get(&task.url);
    if !cfg.user_agent.is_empty() {
        request = request.header(USER_AGENT, cfg.user_agent.clone());
    }
    for (key, value) in &cfg.headers {
        request = request.header(key, value);
    }
    for (key, value) in &task.headers {
        request = request.header(key, value);
    }
    if existing_len > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={}-", existing_len));
    }
    request
}

async fn open_output_writer(
    save_path: &str,
    append: bool,
) -> Result<Box<dyn AsyncWrite + Unpin + Send>, String> {
    let file = if append {
        OpenOptions::new()
            .append(true)
            .open(save_path)
            .await
            .map_err(|e| format!("failed to open {} for append: {}", save_path, e))?
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(save_path)
            .await
            .map_err(|e| format!("failed to open {} for write: {}", save_path, e))?
    };
    Ok(Box::new(file))
}

async fn write_response_body(
    mut response: reqwest::Response,
    mut writer: Box<dyn AsyncWrite + Unpin + Send>,
    pb: Option<&ProgressBar>,
) -> Result<(), String> {
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("failed to read response body: {}", e))?
    {
        writer
            .write_all(&chunk)
            .await
            .map_err(|e| format!("failed to write download output: {}", e))?;
        if let Some(pb) = pb {
            pb.inc(chunk.len() as u64);
        }
    }
    writer
        .flush()
        .await
        .map_err(|e| format!("failed to flush download output: {}", e))?;
    Ok(())
}

async fn build_tasks(
    urls: &[String],
    cfg: &MergedConfig,
    output: Option<&str>,
    output_dir: &str,
    resume: bool,
) -> Result<Vec<DownloadTask>, String> {
    let mut tasks = Vec::new();

    // If -O is specified for single URL, use it as the save path
    let custom_output = if urls.len() == 1 { output } else { None };

    for url in urls {
        let filename = infer_filename(url);
        let (save_path, show_name) = if custom_output == Some("-") {
            // stdout — TLD will write to save_path, we don't use .part
            (filename.clone(), filename.clone())
        } else if let Some(out) = custom_output {
            let part_path = if resume {
                format!("{}.part", out)
            } else {
                out.to_string()
            };
            (part_path, out.to_string())
        } else {
            let base = if output_dir.is_empty() {
                filename.clone()
            } else {
                Path::new(output_dir)
                    .join(&filename)
                    .to_string_lossy()
                    .to_string()
            };
            let part_path = format!("{}.part", base);
            // If .part exists and resume is enabled, use it
            if resume && Path::new(&part_path).exists() {
                (part_path, base)
            } else if resume {
                (part_path, base)
            } else {
                // No resume — use final path directly, .part removed if exists
                let _ = std::fs::remove_file(&part_path);
                (base.clone(), base)
            }
        };

        let task = DownloadTask {
            url: url.clone(),
            save_path,
            show_name,
            id: uuid_v4(),
            headers: build_task_headers(url, cfg).await?,
        };

        tasks.push(task);
    }

    Ok(tasks)
}

async fn build_task_headers(
    url: &str,
    cfg: &MergedConfig,
) -> Result<HashMap<String, String>, String> {
    if cfg
        .headers
        .keys()
        .any(|key| key.eq_ignore_ascii_case("cookie"))
    {
        return Ok(HashMap::new());
    }

    if !matches!(url::Url::parse(url).ok().map(|u| u.scheme().to_string()), Some(scheme) if scheme == "http" || scheme == "https")
    {
        return Ok(HashMap::new());
    }

    let client = build_http_preflight_client(cfg)?;

    let mut request = client.get(url);
    if !cfg.user_agent.is_empty() {
        request = request.header(USER_AGENT, cfg.user_agent.clone());
    }
    for (key, value) in &cfg.headers {
        request = request.header(key, value);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("HTTP preflight failed for {}: {}", url, e))?;

    if !response.status().is_redirection() {
        return Ok(HashMap::new());
    }

    let cookies = extract_cookie_header(response.headers());
    if cookies.is_empty() {
        return Ok(HashMap::new());
    }

    let redirects_to_same_url = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(|location| location == url)
        .unwrap_or(false);

    if !redirects_to_same_url {
        return Ok(HashMap::new());
    }

    let mut headers = HashMap::new();
    headers.insert("Cookie".to_string(), cookies);
    Ok(headers)
}

fn extract_cookie_header(headers: &HeaderMap) -> String {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn infer_filename(url_str: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url_str) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(segment) = segments.last() {
                if !segment.is_empty() {
                    return segment.to_string();
                }
            }
        }
    }
    // Fallback: simple path split for non-URL or edge cases
    if let Some(segment) = url_str.rsplit('/').next() {
        if !segment.is_empty() && !segment.contains('?') && !segment.contains('#') {
            return segment.to_string();
        }
    }
    "index.html".to_string()
}

async fn rename_part_files(urls: &[String], output_dir: &str, output: Option<&str>) {
    for url in urls {
        let filename = infer_filename(url);
        let (part_path, final_path) = if let Some(out) = output {
            if out == "-" {
                continue;
            }
            (format!("{}.part", out), out.to_string())
        } else if output_dir.is_empty() {
            (format!("{}.part", filename), filename)
        } else {
            let base = Path::new(output_dir)
                .join(&filename)
                .to_string_lossy()
                .to_string();
            (format!("{}.part", base), base)
        };

        if Path::new(&part_path).exists() {
            if let Err(e) = std::fs::rename(&part_path, &final_path) {
                eprintln!(
                    "Warning: failed to rename {} → {}: {}",
                    part_path, final_path, e
                );
            }
        }
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}-{:04x}", t, rand_u16())
}

fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (t % 65535) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn extract_cookie_header_collects_cookie_pairs() {
        let mut headers = HeaderMap::new();
        headers.append(SET_COOKIE, HeaderValue::from_static("bcheck=true; Path=/"));
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("session=abc; HttpOnly"),
        );

        assert_eq!(extract_cookie_header(&headers), "bcheck=true; session=abc");
    }
}
