pub fn cn(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|s| !s.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        return input.to_string();
    }
    format!("{}...", &input[..max])
}

pub fn relative_time(input: &str) -> String {
    input.to_string()
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
