pub fn generate_title(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "New Thread".to_string();
    }

    // If it's a question, extract up to the "?"
    if let Some(pos) = trimmed.find('?') {
        let q = &trimmed[..=pos];
        return truncate_title(q);
    }

    truncate_title(trimmed)
}

fn truncate_title(s: &str) -> String {
    let max = 40;
    let s = s.trim();
    // Use first sentence if available
    let s = if let Some(pos) = s.find(". ") {
        &s[..pos]
    } else {
        s
    };
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
