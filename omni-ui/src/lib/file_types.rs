#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    Code,
    Markdown,
    Spreadsheet,
    Document,
    Presentation,
    Image,
    Video,
    Audio,
    Pdf,
    Html,
    Text,
    Binary,
}

pub fn get_file_type(filename: &str) -> FileType {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "md" | "mdx" => FileType::Markdown,
        "xlsx" | "xls" | "xlsm" | "ods" => FileType::Spreadsheet,
        "docx" => FileType::Document,
        "pptx" | "pptm" | "potx" => FileType::Presentation,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" => FileType::Image,
        "mp4" | "webm" | "ogv" | "mov" | "avi" => FileType::Video,
        "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" => FileType::Audio,
        "pdf" => FileType::Pdf,
        "html" | "htm" => FileType::Html,
        "txt" | "log" | "csv" | "tsv" | "xml" | "yaml" | "yml" | "toml" | "env" | "ini" | "cfg"
        | "conf" => FileType::Text,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "rb" | "go" | "java" | "c" | "cpp" | "h"
        | "hpp" | "cs" | "swift" | "kt" | "scala" | "sh" | "bash" | "zsh" | "fish" | "lua"
        | "php" | "r" | "dart" | "ex" | "exs" | "hs" | "clj" | "elm" | "vue" | "svelte" | "css"
        | "scss" | "sass" | "less" | "json" | "jsonc" | "sql" | "dockerfile" | "makefile"
        | "cmake" | "gradle" | "tf" | "hcl" | "proto" | "graphql" | "gql" => FileType::Code,
        _ => FileType::Binary,
    }
}

pub fn ext_to_monaco_language(ext: &str) -> &str {
    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "rb" => "ruby",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "hpp" => "cpp",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" | "zsh" => "shell",
        "lua" => "lua",
        "php" => "php",
        "r" => "r",
        "dart" => "dart",
        "css" | "scss" | "sass" | "less" => "css",
        "json" | "jsonc" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "sql" => "sql",
        "html" | "htm" => "html",
        "graphql" | "gql" => "graphql",
        "proto" => "protobuf",
        "tf" | "hcl" => "hcl",
        "dockerfile" => "dockerfile",
        "md" | "mdx" => "markdown",
        _ => "plaintext",
    }
}

pub fn ext_to_mime_type(ext: &str) -> &str {
    match ext.to_lowercase().as_str() {
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        _ => "application/octet-stream",
    }
}
