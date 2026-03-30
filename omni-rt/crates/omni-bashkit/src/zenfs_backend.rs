use async_trait::async_trait;
use bashkit::{DirEntry, FileType, FsBackend, Metadata, Result};
use futures_channel::oneshot;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

pub struct ZenFsBackend;

macro_rules! bridge {
    ($expr:expr) => {{
        let (tx, rx) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = tx.send($expr);
        });
        rx.await
            .map_err(|_| std::io::Error::other("channel closed"))?
    }};
}

#[async_trait]
impl FsBackend for ZenFsBackend {
    async fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let p = path.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::read_file(&p)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn write(&self, path: &Path, content: &[u8]) -> Result<()> {
        let p = path.to_str().unwrap_or("").to_string();
        let data = content.to_vec();
        Ok(bridge!(omni_zenfs::write_file(&p, &data)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn append(&self, path: &Path, content: &[u8]) -> Result<()> {
        let p = path.to_str().unwrap_or("").to_string();
        let data = content.to_vec();
        Ok(bridge!(omni_zenfs::append_file(&p, &data)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn mkdir(&self, path: &Path, recursive: bool) -> Result<()> {
        let p = path.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::mkdir(&p, recursive)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn remove(&self, path: &Path, recursive: bool) -> Result<()> {
        let p = path.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::remove(&p, recursive)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn stat(&self, path: &Path) -> Result<Metadata> {
        let p = path.to_str().unwrap_or("").to_string();
        let info = bridge!(omni_zenfs::stat(&p)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?;
        Ok(stat_info_to_metadata(&info))
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let p = path.to_str().unwrap_or("").to_string();
        let entries = bridge!(omni_zenfs::read_dir(&p)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?;
        Ok(entries
            .into_iter()
            .map(|e| DirEntry {
                name: e.name,
                metadata: Metadata {
                    file_type: entry_file_type(e.is_file, e.is_dir, e.is_symlink),
                    ..Metadata::default()
                },
            })
            .collect())
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        let p = path.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::exists(&p)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let f = from.to_str().unwrap_or("").to_string();
        let t = to.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::rename(&f, &t)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        let f = from.to_str().unwrap_or("").to_string();
        let t = to.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::copy_file(&f, &t)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn symlink(&self, target: &Path, link: &Path) -> Result<()> {
        let tgt = target.to_str().unwrap_or("").to_string();
        let lnk = link.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::symlink(&tgt, &lnk)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }

    async fn read_link(&self, path: &Path) -> Result<PathBuf> {
        let p = path.to_str().unwrap_or("").to_string();
        let s = bridge!(omni_zenfs::read_link(&p)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?;
        Ok(PathBuf::from(s))
    }

    async fn chmod(&self, path: &Path, mode: u32) -> Result<()> {
        let p = path.to_str().unwrap_or("").to_string();
        Ok(bridge!(omni_zenfs::chmod(&p, mode)
            .await
            .map_err(|e| -> bashkit::Error { e.into() }))?)
    }
}

fn entry_file_type(_is_file: bool, is_dir: bool, is_symlink: bool) -> FileType {
    if is_symlink {
        FileType::Symlink
    } else if is_dir {
        FileType::Directory
    } else {
        FileType::File
    }
}

fn stat_info_to_metadata(info: &omni_zenfs::StatInfo) -> Metadata {
    Metadata {
        file_type: entry_file_type(info.is_file, info.is_dir, info.is_symlink),
        size: info.size,
        mode: info.mode,
        modified: UNIX_EPOCH + Duration::from_millis(info.mtime_ms as u64),
        created: UNIX_EPOCH + Duration::from_millis(info.ctime_ms as u64),
    }
}
