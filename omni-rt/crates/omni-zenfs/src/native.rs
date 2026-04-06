#![cfg(feature = "native")]

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatInfo {
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub mode: u32,
    pub mtime_ms: f64,
    pub atime_ms: f64,
    pub ctime_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntryInfo {
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
}

pub async fn init() -> Result<(), std::io::Error> {
    fs::create_dir_all(root_dir()).await
}

pub async fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    fs::read(resolve_path(path)?).await
}

pub async fn write_file(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    let resolved = resolve_path(path)?;
    ensure_parent(&resolved).await?;
    fs::write(resolved, data).await
}

pub async fn append_file(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    use tokio::io::AsyncWriteExt;

    let resolved = resolve_path(path)?;
    ensure_parent(&resolved).await?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(resolved)
        .await?;
    file.write_all(data).await
}

pub async fn mkdir(path: &str, recursive: bool) -> Result<(), std::io::Error> {
    let resolved = resolve_path(path)?;
    if recursive {
        fs::create_dir_all(resolved).await
    } else {
        fs::create_dir(resolved).await
    }
}

pub async fn remove(path: &str, recursive: bool) -> Result<(), std::io::Error> {
    let resolved = resolve_path(path)?;
    match fs::symlink_metadata(&resolved).await {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            if recursive {
                fs::remove_dir_all(resolved).await
            } else {
                fs::remove_dir(resolved).await
            }
        }
        Ok(_) => fs::remove_file(resolved).await,
        Err(error) => Err(error),
    }
}

pub async fn stat(path: &str) -> Result<StatInfo, std::io::Error> {
    metadata_to_stat(fs::metadata(resolve_path(path)?).await?)
}

pub async fn lstat(path: &str) -> Result<StatInfo, std::io::Error> {
    metadata_to_stat(fs::symlink_metadata(resolve_path(path)?).await?)
}

pub async fn read_dir(path: &str) -> Result<Vec<DirEntryInfo>, std::io::Error> {
    let mut entries = fs::read_dir(resolve_path(path)?).await?;
    let mut out = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        out.push(DirEntryInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_file: file_type.is_file(),
            is_dir: file_type.is_dir(),
            is_symlink: file_type.is_symlink(),
        });
    }

    out.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(out)
}

pub async fn exists(path: &str) -> Result<bool, std::io::Error> {
    match fs::symlink_metadata(resolve_path(path)?).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub async fn rename(from: &str, to: &str) -> Result<(), std::io::Error> {
    let from = resolve_path(from)?;
    let to = resolve_path(to)?;
    ensure_parent(&to).await?;
    fs::rename(from, to).await
}

pub async fn copy_file(from: &str, to: &str) -> Result<(), std::io::Error> {
    let from = resolve_path(from)?;
    let to = resolve_path(to)?;
    ensure_parent(&to).await?;
    fs::copy(from, to).await.map(|_| ())
}

pub async fn symlink(target: &str, path: &str) -> Result<(), std::io::Error> {
    let target = if Path::new(target).is_absolute() {
        resolve_path(target)?
    } else {
        PathBuf::from(target)
    };
    let path = resolve_path(path)?;
    ensure_parent(&path).await?;

    #[cfg(unix)]
    {
        tokio::task::spawn_blocking(move || std::os::unix::fs::symlink(target, path))
            .await
            .map_err(|error| std::io::Error::other(error.to_string()))?
    }

    #[cfg(not(unix))]
    {
        let _ = target;
        let _ = path;
        Err(std::io::Error::other(
            "symlink is not supported on this platform",
        ))
    }
}

pub async fn read_link(path: &str) -> Result<String, std::io::Error> {
    let target = fs::read_link(resolve_path(path)?).await?;
    Ok(virtualize_path(&target))
}

pub async fn chmod(path: &str, mode: u32) -> Result<(), std::io::Error> {
    let resolved = resolve_path(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(resolved, std::fs::Permissions::from_mode(mode)).await
    }

    #[cfg(not(unix))]
    {
        let _ = resolved;
        let _ = mode;
        Ok(())
    }
}

fn metadata_to_stat(metadata: std::fs::Metadata) -> Result<StatInfo, std::io::Error> {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let file_type = metadata.file_type();
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let accessed = metadata.accessed().unwrap_or(modified);
    let created = metadata.created().unwrap_or(modified);

    Ok(StatInfo {
        is_file: file_type.is_file(),
        is_dir: file_type.is_dir(),
        is_symlink: file_type.is_symlink(),
        size: metadata.len(),
        #[cfg(unix)]
        mode: metadata.permissions().mode(),
        #[cfg(not(unix))]
        mode: 0,
        mtime_ms: time_ms(modified),
        atime_ms: time_ms(accessed),
        ctime_ms: time_ms(created),
    })
}

fn time_ms(value: SystemTime) -> f64 {
    value
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or_default()
}

fn root_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".omni").join("data")
}

fn resolve_path(path: &str) -> Result<PathBuf, std::io::Error> {
    let source = Path::new(path);
    if !source.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path must be absolute",
        ));
    }

    let mut resolved = root_dir();
    for component in source.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "path traversal is not allowed",
                ));
            }
            Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "platform path prefixes are not supported",
                ));
            }
        }
    }

    Ok(resolved)
}

async fn ensure_parent(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

fn virtualize_path(path: &Path) -> String {
    let root = root_dir();
    if let Ok(stripped) = path.strip_prefix(&root) {
        let joined = stripped
            .components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/");
        return format!("/{joined}");
    }

    path.to_string_lossy().into_owned()
}
