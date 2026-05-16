use tokio::fs::{File, OpenOptions};
use std::fs::OpenOptions as SyncOpenOptions;
use std::fs::File as SyncFile;

pub async fn create_download_file(
    save_path: &str,
    file_size: Option<i64>,
) -> Result<File, Box<dyn std::error::Error + Send + Sync>> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(save_path).await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    if let Some(size) = file_size {
        let current_len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
        if current_len < size as u64 {
            if let Err(e) = file.set_len(size as u64).await {
                if e.kind() == std::io::ErrorKind::StorageFull {
                    return Err("Insufficient disk space".into());
                }
                if e.kind() == std::io::ErrorKind::FileTooLarge {
                    return Err(
                        "File exceeds filesystem size limit. Please use a filesystem that supports large files."
                            .into(),
                    );
                }
                eprintln!("Warning: Failed to pre-allocate file space ({}), will continue downloading", e);
            }
        }
    }

    Ok(file)
}

pub fn create_download_file_sync(
    save_path: &str,
    file_size: Option<i64>,
) -> Result<SyncFile, String> {
    let file = SyncOpenOptions::new()
        .write(true)
        .create(true)
        .open(save_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    if let Some(size) = file_size {
        let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);
        if current_len < size as u64 {
            if let Err(e) = file.set_len(size as u64) {
                if e.kind() == std::io::ErrorKind::StorageFull {
                    return Err("Insufficient disk space".to_string());
                }
                if e.kind() == std::io::ErrorKind::FileTooLarge {
                    return Err(
                        "File exceeds filesystem size limit. Please use a filesystem that supports large files."
                            .to_string(),
                    );
                }
                eprintln!("Warning: Failed to pre-allocate file space ({}), will continue downloading", e);
            }
        }
    }

    Ok(file)
}
