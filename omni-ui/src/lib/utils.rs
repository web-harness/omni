#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
static DESKTOP_API_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn set_desktop_api_port(port: u16) {
    let _ = DESKTOP_API_PORT.set(port);
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn desktop_api_port() -> u16 {
    *DESKTOP_API_PORT
        .get()
        .expect("desktop api port must be initialized before app launch")
}

pub fn relative_time(input: &str) -> String {
    use chrono::{DateTime, Utc};
    let Ok(then) = DateTime::parse_from_rfc3339(input) else {
        return input.to_string();
    };
    let diff = Utc::now().signed_duration_since(then.with_timezone(&Utc));
    let secs = diff.num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 604800 {
        format!("{}d ago", secs / 86400)
    } else {
        then.format("%b %-d").to_string()
    }
}

pub fn fmt_size(bytes: u64) -> String {
    if bytes < 1000 {
        format!("{}B", bytes)
    } else if bytes < 1_000_000 {
        format!("{:.1}KB", bytes as f64 / 1000.0)
    } else {
        format!("{:.1}MB", bytes as f64 / 1_000_000.0)
    }
}

pub fn file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

#[cfg(all(not(target_arch = "wasm32"), feature = "desktop"))]
pub fn app_url(path: &str) -> String {
    let path = path.trim_start_matches('/');
    format!("http://127.0.0.1:{}/{path}", desktop_api_port())
}

#[cfg(not(all(not(target_arch = "wasm32"), feature = "desktop")))]
pub fn app_url(path: &str) -> String {
    let base_path = option_env!("OMNI_BASE_PATH")
        .unwrap_or("")
        .trim_end_matches('/');
    let path = path.trim_start_matches('/');

    if base_path.is_empty() {
        format!("/{path}")
    } else {
        format!("{base_path}/{path}")
    }
}
