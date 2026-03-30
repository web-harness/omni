use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

mod raw {
    use js_sys::Promise;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn init() -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn readFile(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn writeFile(path: &str, data: &[u8]) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn appendFile(path: &str, data: &[u8]) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, js_name = "mkdir", catch)]
        pub fn mkdir(path: &str, opts: JsValue) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn rm(path: &str, opts: JsValue) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn stat(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn lstat(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn readdir(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn exists(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn rename(from: &str, to: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn copyFile(from: &str, to: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn symlink(target: &str, path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, catch)]
        pub fn readlink(path: &str) -> Result<Promise, JsValue>;

        #[wasm_bindgen(js_namespace = __zenfs, js_name = "chmod", catch)]
        pub fn chmod(path: &str, mode: u32) -> Result<Promise, JsValue>;
    }
}

fn js_err(v: JsValue) -> std::io::Error {
    let msg = v.as_string().unwrap_or_else(|| format!("{v:?}"));
    std::io::Error::other(msg)
}

async fn await_promise(p: Result<Promise, JsValue>) -> Result<JsValue, std::io::Error> {
    JsFuture::from(p.map_err(js_err)?).await.map_err(js_err)
}

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
    await_promise(raw::init()).await?;
    Ok(())
}

pub async fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let val = await_promise(raw::readFile(path)).await?;
    let arr = js_sys::Uint8Array::new(&val);
    Ok(arr.to_vec())
}

pub async fn write_file(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    await_promise(raw::writeFile(path, data)).await?;
    Ok(())
}

pub async fn append_file(path: &str, data: &[u8]) -> Result<(), std::io::Error> {
    await_promise(raw::appendFile(path, data)).await?;
    Ok(())
}

pub async fn mkdir(path: &str, recursive: bool) -> Result<(), std::io::Error> {
    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({ "recursive": recursive }))
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    await_promise(raw::mkdir(path, opts)).await?;
    Ok(())
}

pub async fn remove(path: &str, recursive: bool) -> Result<(), std::io::Error> {
    let opts = serde_wasm_bindgen::to_value(&serde_json::json!({ "recursive": recursive }))
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    await_promise(raw::rm(path, opts)).await?;
    Ok(())
}

pub async fn stat(path: &str) -> Result<StatInfo, std::io::Error> {
    let val = await_promise(raw::stat(path)).await?;
    serde_wasm_bindgen::from_value(val).map_err(|e| std::io::Error::other(e.to_string()))
}

pub async fn lstat(path: &str) -> Result<StatInfo, std::io::Error> {
    let val = await_promise(raw::lstat(path)).await?;
    serde_wasm_bindgen::from_value(val).map_err(|e| std::io::Error::other(e.to_string()))
}

pub async fn read_dir(path: &str) -> Result<Vec<DirEntryInfo>, std::io::Error> {
    let val = await_promise(raw::readdir(path)).await?;
    serde_wasm_bindgen::from_value(val).map_err(|e| std::io::Error::other(e.to_string()))
}

pub async fn exists(path: &str) -> Result<bool, std::io::Error> {
    let val = await_promise(raw::exists(path)).await?;
    Ok(val.as_bool().unwrap_or(false))
}

pub async fn rename(from: &str, to: &str) -> Result<(), std::io::Error> {
    await_promise(raw::rename(from, to)).await?;
    Ok(())
}

pub async fn copy_file(from: &str, to: &str) -> Result<(), std::io::Error> {
    await_promise(raw::copyFile(from, to)).await?;
    Ok(())
}

pub async fn symlink(target: &str, path: &str) -> Result<(), std::io::Error> {
    await_promise(raw::symlink(target, path)).await?;
    Ok(())
}

pub async fn read_link(path: &str) -> Result<String, std::io::Error> {
    let val = await_promise(raw::readlink(path)).await?;
    val.as_string()
        .ok_or_else(|| std::io::Error::other("readlink returned non-string"))
}

pub async fn chmod(path: &str, mode: u32) -> Result<(), std::io::Error> {
    await_promise(raw::chmod(path, mode)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_init() {
        init().await.expect("init failed");
        assert!(exists("/tmp").await.unwrap());
        assert!(exists("/home").await.unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_write_read() {
        init().await.unwrap();
        write_file("/tmp/test_wr.txt", b"hello world")
            .await
            .unwrap();
        let data = read_file("/tmp/test_wr.txt").await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[wasm_bindgen_test]
    async fn test_mkdir_readdir() {
        init().await.unwrap();
        mkdir("/tmp/test_mdir", false).await.unwrap();
        write_file("/tmp/test_mdir/file.txt", b"x").await.unwrap();
        let entries = read_dir("/tmp/test_mdir").await.unwrap();
        assert!(entries.iter().any(|e| e.name == "file.txt"));
    }

    #[wasm_bindgen_test]
    async fn test_stat() {
        init().await.unwrap();
        write_file("/tmp/test_stat.txt", b"hello").await.unwrap();
        let s = stat("/tmp/test_stat.txt").await.unwrap();
        assert!(s.is_file);
        assert_eq!(s.size, 5);
    }

    #[wasm_bindgen_test]
    async fn test_remove() {
        init().await.unwrap();
        write_file("/tmp/test_rm.txt", b"data").await.unwrap();
        remove("/tmp/test_rm.txt", false).await.unwrap();
        assert!(!exists("/tmp/test_rm.txt").await.unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_rename() {
        init().await.unwrap();
        write_file("/tmp/test_rsrc.txt", b"data").await.unwrap();
        rename("/tmp/test_rsrc.txt", "/tmp/test_rdst.txt")
            .await
            .unwrap();
        let data = read_file("/tmp/test_rdst.txt").await.unwrap();
        assert_eq!(data, b"data");
    }

    #[wasm_bindgen_test]
    async fn test_append() {
        init().await.unwrap();
        write_file("/tmp/test_app.txt", b"hello ").await.unwrap();
        append_file("/tmp/test_app.txt", b"world").await.unwrap();
        let data = read_file("/tmp/test_app.txt").await.unwrap();
        assert_eq!(data, b"hello world");
    }

    #[wasm_bindgen_test]
    async fn test_symlink_readlink() {
        init().await.unwrap();
        write_file("/tmp/test_stgt.txt", b"x").await.unwrap();
        symlink("/tmp/test_stgt.txt", "/tmp/test_slnk.txt")
            .await
            .unwrap();
        let target = read_link("/tmp/test_slnk.txt").await.unwrap();
        assert_eq!(target, "/tmp/test_stgt.txt");
    }

    #[wasm_bindgen_test]
    async fn test_copy() {
        init().await.unwrap();
        write_file("/tmp/test_csrc.txt", b"copied").await.unwrap();
        copy_file("/tmp/test_csrc.txt", "/tmp/test_cdst.txt")
            .await
            .unwrap();
        let data = read_file("/tmp/test_cdst.txt").await.unwrap();
        assert_eq!(data, b"copied");
    }

    #[wasm_bindgen_test]
    async fn test_chmod() {
        init().await.unwrap();
        write_file("/tmp/test_chm.txt", b"x").await.unwrap();
        chmod("/tmp/test_chm.txt", 0o755).await.unwrap();
        let s = stat("/tmp/test_chm.txt").await.unwrap();
        assert_eq!(s.mode & 0o777, 0o755);
    }

    #[wasm_bindgen_test]
    async fn test_error_not_found() {
        init().await.unwrap();
        let result = read_file("/tmp/nonexistent_surely.txt").await;
        assert!(result.is_err());
    }
}
