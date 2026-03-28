use std::env;
use std::fs;
use std::path::PathBuf;

const SPEC_URL: &str = "https://langchain-ai.github.io/agent-protocol/openapi.json";
const SPEC_FILE: &str = "specs/agent-protocol.json";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={SPEC_FILE}");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let spec_path = manifest_dir.join(SPEC_FILE);

    if let Some(parent) = spec_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // Download if missing
    if !spec_path.exists() {
        eprintln!("Downloading agent-protocol spec from {SPEC_URL}");
        let body = reqwest::blocking::get(SPEC_URL)
            .unwrap_or_else(|e| panic!("Failed to download spec from {SPEC_URL}: {e}"))
            .error_for_status()
            .unwrap_or_else(|e| panic!("HTTP error downloading spec: {e}"))
            .text()
            .unwrap_or_else(|e| panic!("Failed to read response body: {e}"));
        fs::write(&spec_path, &body)
            .unwrap_or_else(|e| panic!("Failed to write {}: {e}", spec_path.display()));
        eprintln!("Downloaded spec to {}", spec_path.display());
    }

    // Validate JSON
    let contents = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", spec_path.display()));
    let doc: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Invalid JSON in {}: {e}", spec_path.display()));

    // Check it's an OpenAPI doc
    assert!(
        doc.get("openapi").is_some(),
        "{SPEC_FILE} is not an OpenAPI document (missing 'openapi' field)"
    );
}
