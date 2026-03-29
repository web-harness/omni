include!(concat!(env!("OUT_DIR"), "/fixtures.rs"));

pub enum FixtureContent {
    Text(&'static str),
    Base64(&'static str),
}

pub fn get_fixture(path: &str) -> FixtureContent {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => FixtureContent::Text(include_str!("../../fixtures/sample.rs")),
        "js" | "jsx" => FixtureContent::Text(include_str!("../../fixtures/sample.js")),
        "ts" | "tsx" => FixtureContent::Text(include_str!("../../fixtures/sample.ts")),
        "py" => FixtureContent::Text(include_str!("../../fixtures/sample.py")),
        "md" | "mdx" => FixtureContent::Text(include_str!("../../fixtures/sample.md")),
        "html" | "htm" => FixtureContent::Text(include_str!("../../fixtures/sample.html")),
        "css" | "scss" | "sass" => FixtureContent::Text(include_str!("../../fixtures/sample.css")),
        "json" => FixtureContent::Text(include_str!("../../fixtures/sample.json")),
        "txt" => FixtureContent::Text(include_str!("../../fixtures/sample.txt")),
        "toml" => FixtureContent::Text(include_str!("../../fixtures/sample.toml")),
        "sh" | "bash" => FixtureContent::Text(include_str!("../../fixtures/sample.sh")),
        "svg" => FixtureContent::Text(include_str!("../../fixtures/sample.svg")),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" => {
            FixtureContent::Base64(FIXTURE_PNG_B64)
        }
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => FixtureContent::Base64(FIXTURE_WAV_B64),
        "mp4" | "webm" | "ogv" | "mov" | "avi" => {
            // No real video fixture — return empty so the viewer shows a placeholder
            FixtureContent::Base64("")
        }
        "pdf" => FixtureContent::Base64(FIXTURE_PDF_B64),
        _ => FixtureContent::Text(""),
    }
}

pub fn fixture_text(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::Text(s) => s.to_string(),
        FixtureContent::Base64(_) => String::new(),
    }
}

pub fn fixture_b64(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::Base64(s) => s.to_string(),
        FixtureContent::Text(_) => String::new(),
    }
}
