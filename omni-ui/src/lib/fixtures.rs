pub enum FixtureContent {
    Text(&'static str),
    SourcePath(&'static str),
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
            FixtureContent::SourcePath("fixtures/sample.png")
        }
        "wav" | "mp3" | "ogg" | "flac" | "aac" | "m4a" => {
            FixtureContent::SourcePath("fixtures/sample.wav")
        }
        "mp4" | "webm" | "ogv" | "mov" | "avi" => FixtureContent::SourcePath("fixtures/sample.mp4"),
        "pdf" => FixtureContent::SourcePath("fixtures/sample.pdf"),
        "xlsx" | "xls" | "xlsm" | "ods" => FixtureContent::SourcePath("fixtures/sample.xlsx"),
        "docx" => FixtureContent::SourcePath("fixtures/sample.docx"),
        "pptx" | "pptm" | "potx" => FixtureContent::SourcePath("fixtures/sample.pptx"),
        _ => FixtureContent::Text(""),
    }
}

pub fn fixture_text(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::Text(s) => s.to_string(),
        FixtureContent::SourcePath(_) => String::new(),
    }
}

pub fn fixture_url(path: &str) -> String {
    match get_fixture(path) {
        FixtureContent::SourcePath(path) => crate::lib::utils::api_url(path),
        FixtureContent::Text(_) => String::new(),
    }
}
