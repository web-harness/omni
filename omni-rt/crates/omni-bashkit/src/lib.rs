#[cfg(target_arch = "wasm32")]
pub mod zenfs_backend;
pub use bashkit::{Bash, BashTool, ExecutionLimits};

#[cfg(target_arch = "wasm32")]
use bashkit::PosixFs;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
pub use zenfs_backend::ZenFsBackend;

#[cfg(target_arch = "wasm32")]
pub fn build_bash() -> Bash {
    let fs = Arc::new(PosixFs::new(ZenFsBackend));
    Bash::builder().fs(fs).build()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn build_bash() -> Bash {
    let workspace_root = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".omni")
        .join("data")
        .join("home")
        .join("workspace");

    let _ = std::fs::create_dir_all(&workspace_root);

    Bash::builder()
        .mount_real_readwrite_at(workspace_root, "/home/workspace")
        .build()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn execute(command: String, cwd: Option<String>) -> Result<JsValue, JsValue> {
    let mut bash = build_bash();
    let cwd = cwd
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/home/workspace".to_string());
    let script = format!("cd \"{}\" && {}", cwd.replace('"', "\\\""), command);

    let obj = js_sys::Object::new();
    match bash.exec(&script).await {
        Ok(result) => {
            let mut output = result.stdout;
            if !result.stderr.is_empty() {
                output.push_str(&result.stderr);
            }
            js_sys::Reflect::set(&obj, &"output".into(), &output.into())?;
            js_sys::Reflect::set(
                &obj,
                &"exitCode".into(),
                &JsValue::from_f64(result.exit_code as f64),
            )?;
            js_sys::Reflect::set(
                &obj,
                &"truncated".into(),
                &JsValue::from_bool(result.stdout_truncated || result.stderr_truncated),
            )?;
        }
        Err(err) => {
            js_sys::Reflect::set(&obj, &"output".into(), &format!("Error: {err}").into())?;
            js_sys::Reflect::set(&obj, &"exitCode".into(), &JsValue::from_f64(1.0))?;
            js_sys::Reflect::set(&obj, &"truncated".into(), &JsValue::from_bool(false))?;
        }
    }

    Ok(obj.into())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn execute_native(
    command: String,
    cwd: Option<String>,
) -> Result<(String, i32, bool), std::io::Error> {
    let mut bash = build_bash();
    let cwd = cwd
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "/home/workspace".to_string());
    let script = format!("cd \"{}\" && {}", cwd.replace('"', "\\\""), command);
    let result = bash
        .exec(&script)
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let mut output = result.stdout;
    if !result.stderr.is_empty() {
        output.push_str(&result.stderr);
    }

    Ok((
        output,
        result.exit_code,
        result.stdout_truncated || result.stderr_truncated,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn bash() -> Bash {
        omni_zenfs::init().await.expect("zenfs init failed");
        build_bash()
    }

    #[wasm_bindgen_test]
    async fn test_bash_echo() {
        let mut b = bash().await;
        let result = b.exec("echo hello").await.unwrap();
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[wasm_bindgen_test]
    async fn test_bash_write_read() {
        let mut b = bash().await;
        b.exec("echo data > /tmp/bk_test.txt").await.unwrap();
        let result = b.exec("cat /tmp/bk_test.txt").await.unwrap();
        assert_eq!(result.stdout.trim(), "data");
    }

    #[wasm_bindgen_test]
    async fn test_bash_mkdir_ls() {
        let mut b = bash().await;
        b.exec("mkdir -p /tmp/bk_dir").await.unwrap();
        let result = b.exec("ls /tmp").await.unwrap();
        assert!(result.stdout.contains("bk_dir"));
    }

    #[wasm_bindgen_test]
    async fn test_bash_pipe() {
        let mut b = bash().await;
        let result = b.exec("echo hello | cat").await.unwrap();
        assert_eq!(result.stdout.trim(), "hello");
    }

    #[wasm_bindgen_test]
    async fn test_bash_env() {
        let mut b = bash().await;
        let result = b.exec("export X=42 && echo $X").await.unwrap();
        assert_eq!(result.stdout.trim(), "42");
    }
}
