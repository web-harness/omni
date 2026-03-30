pub mod zenfs_backend;
pub use bashkit::{Bash, BashTool, ExecutionLimits};
pub use zenfs_backend::ZenFsBackend;

use bashkit::PosixFs;
use std::sync::Arc;

pub fn build_bash() -> Bash {
    let fs = Arc::new(PosixFs::new(ZenFsBackend));
    Bash::builder().fs(fs).build()
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
