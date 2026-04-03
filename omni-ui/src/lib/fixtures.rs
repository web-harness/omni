use dioxus::prelude::*;

const FIXTURE_PNG: Asset = asset!("/fixtures/sample.png");
const FIXTURE_WAV: Asset = asset!("/fixtures/sample.wav");
const FIXTURE_PDF: Asset = asset!("/fixtures/sample.pdf");
const FIXTURE_MP4: Asset = asset!("/fixtures/sample.mp4");
const FIXTURE_XLSX: Asset = asset!("/fixtures/sample.xlsx");
const FIXTURE_DOCX: Asset = asset!("/fixtures/sample.docx");
const FIXTURE_PPTX: Asset = asset!("/fixtures/sample.pptx");

pub enum FixtureContent {
    Text(&'static str),
    SourceUrl(Asset),
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
            FixtureContent::SourceUrl(FIXTURE_PNG)
        }
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => FixtureContent::SourceUrl(FIXTURE_WAV),
        "mp4" | "webm" | "ogv" | "mov" | "avi" => FixtureContent::SourceUrl(FIXTURE_MP4),
        "pdf" => FixtureContent::SourceUrl(FIXTURE_PDF),
        "xlsx" | "xls" | "xlsm" | "ods" => FixtureContent::SourceUrl(FIXTURE_XLSX),
        "docx" => FixtureContent::SourceUrl(FIXTURE_DOCX),
        "pptx" | "pptm" | "potx" => FixtureContent::SourceUrl(FIXTURE_PPTX),
        _ => FixtureContent::Text(""),
    }
}

pub fn fixture_text(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::Text(s) => s.to_string(),
        FixtureContent::SourceUrl(_) => String::new(),
    }
}

pub fn fixture_url(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::SourceUrl(asset) => asset.to_string(),
        FixtureContent::Text(_) => String::new(),
    }
}
