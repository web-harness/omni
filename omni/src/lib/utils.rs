use chrono::Local;

pub fn format_timestamp(timestamp: &str) -> String {
    timestamp.chars().take(10).collect::<String>()
}

pub fn relative_time(timestamp: &str) -> String {
    format!("just now")
}

pub mod css_utils {
    pub fn cn(classes: &[&str]) -> String {
        classes.join(" ")
    }
}
