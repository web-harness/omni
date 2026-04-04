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
