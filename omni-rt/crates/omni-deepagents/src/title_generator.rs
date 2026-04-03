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

#[cfg(test)]
mod tests {
    use super::generate_title;

    #[test]
    fn empty_input_falls_back_to_new_thread() {
        assert_eq!(generate_title("   "), "New Thread");
    }

    #[test]
    fn question_prefers_question_boundary() {
        let title = generate_title("What is the best way to handle optimistic locking in sqlite?");
        assert_eq!(title, "What is the best way to handle optimisti...");
    }

    #[test]
    fn long_text_is_truncated() {
        let title = generate_title("Implement a complete service worker architecture with checkpoint persistence and sandboxed execution support");
        assert_eq!(title, "Implement a complete service worker arch...");
    }
}
