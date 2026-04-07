use omni_zenfs as zenfs;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct WorkspaceSeedEntry {
    pub path: String,
    pub text: Option<&'static str>,
    pub fixture: Option<&'static str>,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceSeedEntryView {
    pub path: String,
    pub text: Option<String>,
    pub fixture: Option<String>,
    pub size: u64,
}

#[derive(Clone, Copy)]
struct RelativeSeedEntry {
    path: &'static str,
    text: Option<&'static str>,
    fixture: Option<&'static str>,
    size: u64,
}

const TEST_WORKSPACE_RELATIVE: &[RelativeSeedEntry] = &[
    RelativeSeedEntry {
        path: "README.md",
        text: Some("# Workspace\n"),
        fixture: None,
        size: 12,
    },
    RelativeSeedEntry {
        path: "public/app.js",
        text: Some("console.log('app');\n"),
        fixture: None,
        size: 2600,
    },
    RelativeSeedEntry {
        path: "public/index.html",
        text: Some("<!doctype html><html><body>test</body></html>\n"),
        fixture: None,
        size: 6900,
    },
    RelativeSeedEntry {
        path: "public/styles.css",
        text: Some("body { font-family: monospace; }\n"),
        fixture: None,
        size: 3400,
    },
    RelativeSeedEntry {
        path: "scripts/flush_todos.js",
        text: Some("#!/usr/bin/env node\nconsole.log('flush todos');\n"),
        fixture: None,
        size: 381,
    },
    RelativeSeedEntry {
        path: "server/server.js",
        text: Some("export const server = true;\n"),
        fixture: None,
        size: 850,
    },
    RelativeSeedEntry {
        path: "server/todos.json",
        text: Some("{\n  \"todos\": []\n}\n"),
        fixture: None,
        size: 314,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.js",
        text: None,
        fixture: Some("sample.js"),
        size: 169,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.rs",
        text: None,
        fixture: Some("sample.rs"),
        size: 512,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.ts",
        text: None,
        fixture: Some("sample.ts"),
        size: 620,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.py",
        text: None,
        fixture: Some("sample.py"),
        size: 580,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.sh",
        text: None,
        fixture: Some("sample.sh"),
        size: 410,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.md",
        text: None,
        fixture: Some("sample.md"),
        size: 740,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.html",
        text: None,
        fixture: Some("sample.html"),
        size: 890,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.css",
        text: None,
        fixture: Some("sample.css"),
        size: 660,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.json",
        text: None,
        fixture: Some("sample.json"),
        size: 520,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.toml",
        text: None,
        fixture: Some("sample.toml"),
        size: 280,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.txt",
        text: None,
        fixture: Some("sample.txt"),
        size: 940,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.svg",
        text: None,
        fixture: Some("sample.svg"),
        size: 480,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.png",
        text: None,
        fixture: Some("sample.png"),
        size: 120,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.jpg",
        text: None,
        fixture: Some("sample.jpg"),
        size: 120,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.pdf",
        text: None,
        fixture: Some("sample.pdf"),
        size: 800,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.xlsx",
        text: None,
        fixture: Some("sample.xlsx"),
        size: 2138,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.docx",
        text: None,
        fixture: Some("sample.docx"),
        size: 1672,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.pptx",
        text: None,
        fixture: Some("sample.pptx"),
        size: 5605,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.wav",
        text: None,
        fixture: Some("sample.wav"),
        size: 46,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.mp3",
        text: None,
        fixture: Some("sample.mp3"),
        size: 64,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.mp4",
        text: None,
        fixture: Some("sample.mp4"),
        size: 256,
    },
    RelativeSeedEntry {
        path: "fixtures/sample.bin",
        text: None,
        fixture: Some("sample.bin"),
        size: 128,
    },
];

pub const DEFAULT_WORKSPACE_ORDER: [&str; 3] = [
    "/home/user/projects/test",
    "/home/user/projects/omni",
    "/home/user/projects/omni-rt",
];

pub fn default_workspace_for_index(index: usize) -> &'static str {
    match index {
        1 => DEFAULT_WORKSPACE_ORDER[1],
        2 => DEFAULT_WORKSPACE_ORDER[2],
        _ => DEFAULT_WORKSPACE_ORDER[0],
    }
}

pub fn workspace_seed_entries() -> Vec<WorkspaceSeedEntry> {
    let mut entries = Vec::new();

    for root in ["/home/workspace", "/home/user/projects/test"] {
        for entry in TEST_WORKSPACE_RELATIVE {
            entries.push(WorkspaceSeedEntry {
                path: format!("{root}/{}", entry.path),
                text: entry.text,
                fixture: entry.fixture,
                size: entry.size,
            });
        }
    }

    entries.extend([
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/src/main.rs".to_string(),
            text: Some("fn main() {}\n"),
            fixture: None,
            size: 9612,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/src/components/chat/mod.rs".to_string(),
            text: Some("pub fn chat() {}\n"),
            fixture: None,
            size: 14020,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/src/components/sidebar/mod.rs".to_string(),
            text: Some("pub fn sidebar() {}\n"),
            fixture: None,
            size: 3400,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/src/lib/mod.rs".to_string(),
            text: Some("pub mod sample;\n"),
            fixture: None,
            size: 7903,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/Cargo.toml".to_string(),
            text: Some("[package]\nname = \"omni\"\n"),
            fixture: None,
            size: 1200,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni/README.md".to_string(),
            text: Some("# omni\n"),
            fixture: None,
            size: 4089,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni-rt/crates/omni-protocol/src/lib.rs".to_string(),
            text: Some("pub struct Protocol;\n"),
            fixture: None,
            size: 5120,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni-rt/crates/omni-rt/src/main.rs".to_string(),
            text: Some("fn main() {}\n"),
            fixture: None,
            size: 3800,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni-rt/crates/omni-dock/src/omni-dock.ts".to_string(),
            text: Some("export const dock = true;\n"),
            fixture: None,
            size: 8200,
        },
        WorkspaceSeedEntry {
            path: "/home/user/projects/omni-rt/Cargo.toml".to_string(),
            text: Some("[workspace]\n"),
            fixture: None,
            size: 980,
        },
    ]);

    entries
}

pub fn workspace_seed_entry_views() -> Vec<WorkspaceSeedEntryView> {
    workspace_seed_entries()
        .into_iter()
        .map(|entry| WorkspaceSeedEntryView {
            path: entry.path,
            text: entry.text.map(str::to_string),
            fixture: entry.fixture.map(str::to_string),
            size: entry.size,
        })
        .collect()
}

pub async fn ensure_workspace_scaffold() -> Result<(), std::io::Error> {
    for entry in workspace_seed_entries() {
        if zenfs::exists(&entry.path).await? {
            continue;
        }
        if let Some((parent, _)) = entry.path.rsplit_once('/') {
            zenfs::mkdir(parent, true).await?;
        }
        zenfs::write_file(&entry.path, &seed_bytes(&entry)?).await?;
    }
    Ok(())
}

pub fn seeded_size(path: &str) -> Option<u64> {
    workspace_seed_entries()
        .into_iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.size)
}

fn seed_bytes(entry: &WorkspaceSeedEntry) -> Result<Vec<u8>, std::io::Error> {
    if let Some(text) = entry.text {
        return Ok(text.as_bytes().to_vec());
    }

    if let Some(fixture) = entry.fixture {
        return fixture_bytes_by_name(fixture, entry.size);
    }

    Err(std::io::Error::other(format!(
        "unknown workspace seed entry: {}",
        entry.path
    )))
}

fn fixture_bytes_by_name(name: &str, size: u64) -> Result<Vec<u8>, std::io::Error> {
    let bytes = match name {
        "sample.rs" => include_str!("../../../../omni-ui/fixtures/sample.rs")
            .as_bytes()
            .to_vec(),
        "sample.js" => include_str!("../../../../omni-ui/fixtures/sample.js")
            .as_bytes()
            .to_vec(),
        "sample.ts" => include_str!("../../../../omni-ui/fixtures/sample.ts")
            .as_bytes()
            .to_vec(),
        "sample.py" => include_str!("../../../../omni-ui/fixtures/sample.py")
            .as_bytes()
            .to_vec(),
        "sample.sh" => include_str!("../../../../omni-ui/fixtures/sample.sh")
            .as_bytes()
            .to_vec(),
        "sample.md" => include_str!("../../../../omni-ui/fixtures/sample.md")
            .as_bytes()
            .to_vec(),
        "sample.html" => include_str!("../../../../omni-ui/fixtures/sample.html")
            .as_bytes()
            .to_vec(),
        "sample.css" => include_str!("../../../../omni-ui/fixtures/sample.css")
            .as_bytes()
            .to_vec(),
        "sample.json" => include_str!("../../../../omni-ui/fixtures/sample.json")
            .as_bytes()
            .to_vec(),
        "sample.toml" => include_str!("../../../../omni-ui/fixtures/sample.toml")
            .as_bytes()
            .to_vec(),
        "sample.txt" => include_str!("../../../../omni-ui/fixtures/sample.txt")
            .as_bytes()
            .to_vec(),
        "sample.svg" => include_str!("../../../../omni-ui/fixtures/sample.svg")
            .as_bytes()
            .to_vec(),
        "sample.png" => include_bytes!("../../../../omni-ui/fixtures/sample.png").to_vec(),
        "sample.jpg" => repeated_bytes(b"jpg\n", size),
        "sample.pdf" => include_bytes!("../../../../omni-ui/fixtures/sample.pdf").to_vec(),
        "sample.xlsx" => include_bytes!("../../../../omni-ui/fixtures/sample.xlsx").to_vec(),
        "sample.docx" => include_bytes!("../../../../omni-ui/fixtures/sample.docx").to_vec(),
        "sample.pptx" => include_bytes!("../../../../omni-ui/fixtures/sample.pptx").to_vec(),
        "sample.wav" => include_bytes!("../../../../omni-ui/fixtures/sample.wav").to_vec(),
        "sample.mp3" => repeated_bytes(b"ID3\n", size),
        "sample.mp4" => include_bytes!("../../../../omni-ui/fixtures/sample.mp4").to_vec(),
        "sample.bin" => repeated_bytes(b"bin\n", size),
        _ => {
            return Err(std::io::Error::other(format!(
                "unknown workspace fixture: {name}"
            )))
        }
    };
    Ok(bytes)
}

fn repeated_bytes(seed: &[u8], size: u64) -> Vec<u8> {
    let target = size as usize;
    let mut out = Vec::with_capacity(target);
    while out.len() < target {
        let remaining = target - out.len();
        if remaining >= seed.len() {
            out.extend_from_slice(seed);
        } else {
            out.extend_from_slice(&seed[..remaining]);
        }
    }
    out
}
