use base64::{engine::general_purpose::STANDARD, Engine as _};

const FIXTURE_PNG_BYTES: &[u8] = include_bytes!("../../fixtures/sample.png");
const FIXTURE_WAV_BYTES: &[u8] = include_bytes!("../../fixtures/sample.wav");
const FIXTURE_PDF_BYTES: &[u8] = include_bytes!("../../fixtures/sample.pdf");
const FIXTURE_MP4_BYTES: &[u8] = include_bytes!("../../fixtures/sample.mp4");

pub enum FixtureContent {
    Text(&'static str),
    Base64,
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
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" => FixtureContent::Base64,
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => FixtureContent::Base64,
        "mp4" | "webm" | "ogv" | "mov" | "avi" => FixtureContent::Base64,
        "pdf" => FixtureContent::Base64,
        _ => FixtureContent::Text(""),
    }
}

pub fn fixture_text(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::Text(s) => s.to_string(),
        FixtureContent::Base64 => String::new(),
    }
}

pub fn fixture_b64(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" => {
            STANDARD.encode(FIXTURE_PNG_BYTES)
        }
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => STANDARD.encode(FIXTURE_WAV_BYTES),
        "mp4" | "webm" | "ogv" | "mov" | "avi" => STANDARD.encode(FIXTURE_MP4_BYTES),
        "pdf" => STANDARD.encode(FIXTURE_PDF_BYTES),
        _ => String::new(),
    }
}
