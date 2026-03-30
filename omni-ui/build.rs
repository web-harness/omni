use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let workspace = manifest_dir.parent().unwrap();

    generate_fixtures(&out_dir);

    let wcs = [
        ("omni-rt/crates/omni-dock", "omni-dock.js", None),
        ("omni-rt/crates/omni-popper", "omni-popper.js", None),
        ("omni-rt/crates/omni-monaco", "omni-monaco.js", None),
        ("omni-rt/crates/omni-mdx", "omni-mdx.js", None),
        (
            "omni-rt/crates/omni-pdfjs",
            "omni-pdfjs.js",
            Some("omni-pdfjs.worker.js"),
        ),
        ("omni-rt/crates/omni-plyr", "omni-plyr.js", None),
        ("omni-rt/crates/omni-zenfs", "omni-zenfs.js", None),
    ];

    for (crate_rel, js_name, extra_file) in &wcs {
        let crate_dir = workspace.join(crate_rel);

        println!("cargo:rerun-if-changed={}", crate_dir.join("src").display());
        println!(
            "cargo:rerun-if-changed={}",
            crate_dir.join("package.json").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            crate_dir.join("tsconfig.json").display()
        );

        let status = Command::new("npm")
            .args(["run", "build"])
            .current_dir(&crate_dir)
            .status()
            .unwrap_or_else(|e| panic!("failed to run npm in {crate_rel}: {e}"));

        assert!(status.success(), "npm run build failed in {crate_rel}");

        let public_dir = manifest_dir.join("public");
        fs::create_dir_all(&public_dir)
            .unwrap_or_else(|e| panic!("failed to create public dir: {e}"));
        fs::copy(
            crate_dir.join("dist").join(js_name),
            public_dir.join(js_name),
        )
        .unwrap_or_else(|e| panic!("failed to copy {js_name}: {e}"));

        if let Some(extra) = extra_file {
            fs::copy(crate_dir.join("dist").join(extra), public_dir.join(extra))
                .unwrap_or_else(|e| panic!("failed to copy {extra}: {e}"));
        }
    }
}

fn generate_fixtures(out_dir: &PathBuf) {
    let png_b64 = b64(&make_png());
    let wav_b64 = b64(&make_wav());
    let pdf_b64 = b64(&make_pdf());
    let mp4_b64 = b64(&make_mp4());

    let code = format!(
        "pub const FIXTURE_PNG_B64: &str = \"{png_b64}\";\n\
         pub const FIXTURE_WAV_B64: &str = \"{wav_b64}\";\n\
         pub const FIXTURE_PDF_B64: &str = \"{pdf_b64}\";\n\
         pub const FIXTURE_MP4_B64: &str = \"{mp4_b64}\";\n"
    );
    fs::write(out_dir.join("fixtures.rs"), code).expect("failed to write fixtures.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let fixtures_dir = manifest_dir.join("fixtures");
    fs::create_dir_all(&fixtures_dir).expect("failed to create fixtures dir");

    let text_fixtures: &[(&str, &str)] = &[
        ("sample.rs", r#"use std::collections::HashMap;

fn main() {
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("hello".to_string(), 42);
    for (key, value) in &map {
        println!("{}: {}", key, value);
    }
}
"#),
        ("sample.js", r#"function greet(name) {
  return `Hello, ${name}!`;
}

const users = ["Alice", "Bob", "Charlie"];
users.forEach(user => console.log(greet(user)));

export default greet;
"#),
        ("sample.ts", r#"interface User {
  id: number;
  name: string;
  email: string;
}

function greet(user: User): string {
  return `Hello, ${user.name}!`;
}

const user: User = { id: 1, name: "Alice", email: "alice@example.com" };
console.log(greet(user));
"#),
        ("sample.py", r#"from dataclasses import dataclass
from typing import List

@dataclass
class User:
    id: int
    name: str
    email: str

def greet(user: User) -> str:
    return f"Hello, {user.name}!"

users: List[User] = [
    User(1, "Alice", "alice@example.com"),
    User(2, "Bob", "bob@example.com"),
]

for user in users:
    print(greet(user))
"#),
        ("sample.md", r#"# Sample Document

This is a **sample markdown** file.

## Features

- Item one
- Item two
- Item three

## Code

```rust
fn main() {
    println!("Hello, world!");
}
```
"#),
        ("sample.html", r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Sample Page</title>
</head>
<body>
  <h1>Hello, World!</h1>
  <p>This is a sample HTML file.</p>
</body>
</html>
"#),
        ("sample.css", r##":root {
  --primary: #3b82f6;
  --bg: #ffffff;
}

body {
  font-family: sans-serif;
  background: var(--bg);
  color: #111;
  margin: 0;
  padding: 1rem;
}

h1 {
  color: var(--primary);
}
"##),
        ("sample.json", r#"{
  "name": "sample",
  "version": "1.0.0",
  "description": "A sample JSON file",
  "tags": ["example", "demo"],
  "meta": {
    "author": "Omni",
    "created": "2026-01-01"
  }
}
"#),
        ("sample.txt", "This is a sample plain text file.\n\nIt contains multiple lines of text.\nYou can use it to test text rendering.\n\nThe quick brown fox jumps over the lazy dog.\n"),
        ("sample.toml", r#"[package]
name = "sample"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }

[profile.release]
opt-level = "s"
"#),
        ("sample.sh", r#"#!/usr/bin/env bash
set -euo pipefail

NAME="${1:-World}"
echo "Hello, ${NAME}!"

for i in 1 2 3; do
  echo "  Item $i"
done
"#),
        ("sample.svg", r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40" fill="#3b82f6" />
  <text x="50" y="55" text-anchor="middle" fill="white" font-size="14">SVG</text>
</svg>
"##),
    ];

    for (name, content) in text_fixtures {
        let path = fixtures_dir.join(name);
        if !path.exists() {
            fs::write(&path, content).unwrap_or_else(|e| panic!("failed to write {name}: {e}"));
        }
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

/// 8×8 RGB PNG — a simple colour gradient block.
fn make_png() -> Vec<u8> {
    // --- helpers ---
    fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFF_FFFF;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB8_8320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }
    fn adler32(data: &[u8]) -> u32 {
        let (mut s1, mut s2): (u32, u32) = (1, 0);
        for &b in data {
            s1 = (s1 + b as u32) % 65521;
            s2 = (s2 + s1) % 65521;
        }
        (s2 << 16) | s1
    }
    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut c = Vec::new();
        c.extend_from_slice(&(data.len() as u32).to_be_bytes());
        c.extend_from_slice(tag);
        c.extend_from_slice(data);
        let mut tagged = tag.to_vec();
        tagged.extend_from_slice(data);
        c.extend_from_slice(&crc32(&tagged).to_be_bytes());
        c
    }
    fn deflate_store(data: &[u8]) -> Vec<u8> {
        // zlib header (CM=8, CINFO=1 → window=256 bytes, FCHECK so header%31==0)
        let mut out = vec![0x78, 0x01];
        // DEFLATE non-compressed block header
        let len = data.len() as u16;
        out.push(0x01); // BFINAL=1, BTYPE=00 (no compression)
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(data);
        // adler32 checksum
        out.extend_from_slice(&adler32(data).to_be_bytes());
        out
    }

    const W: u32 = 8;
    const H: u32 = 8;
    // IHDR
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&W.to_be_bytes());
    ihdr_data.extend_from_slice(&H.to_be_bytes());
    ihdr_data.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit RGB

    let mut scanlines: Vec<u8> = Vec::new();
    for y in 0..H {
        scanlines.push(0); // filter: None
        for x in 0..W {
            let r = (x * 255 / (W - 1)) as u8;
            let g = (y * 255 / (H - 1)) as u8;
            let b = 128u8.wrapping_add((x + y) as u8 * 16);
            scanlines.push(r);
            scanlines.push(g);
            scanlines.push(b);
        }
    }
    let idat_data = deflate_store(&scanlines);

    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    out.extend(chunk(b"IHDR", &ihdr_data));
    out.extend(chunk(b"IDAT", &idat_data));
    out.extend(chunk(b"IEND", &[]));
    out
}

/// 1-second 440 Hz sine wave, 44100 Hz, 16-bit mono WAV.
fn make_wav() -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let channels: u16 = 1;
    let bits: u16 = 16;
    let freq = 440.0_f64;
    let duration_secs = 1.0_f64;
    let n_samples = (sample_rate as f64 * duration_secs) as usize;

    let mut samples: Vec<u8> = Vec::with_capacity(n_samples * 2);
    for i in 0..n_samples {
        let t = i as f64 / sample_rate as f64;
        // fade out over last 10% to avoid click
        let env = if t > 0.9 { (1.0 - t) / 0.1 } else { 1.0 };
        let s = (env * 0.6 * (2.0 * std::f64::consts::PI * freq * t).sin()) * 32767.0;
        let s16 = s.round() as i16;
        samples.extend_from_slice(&s16.to_le_bytes());
    }

    let data_len = samples.len() as u32;
    let byte_rate = sample_rate * channels as u32 * bits as u32 / 8;
    let block_align = channels * bits / 8;

    let mut out = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend(samples);
    out
}

/// Minimal PDF 1.4 with a visible "Sample PDF" text page.
fn make_pdf() -> Vec<u8> {
    let stream_content = b"BT /F1 18 Tf 72 700 Td (Sample PDF Fixture) Tj ET";
    let stream_len = stream_content.len();

    let objects: &[&[u8]] = &[
        b"1 0 obj\n<</Type /Catalog /Pages 2 0 R>>\nendobj",
        b"2 0 obj\n<</Type /Pages /Kids [3 0 R] /Count 1>>\nendobj",
        &format!(
            "3 0 obj\n<</Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
             /Contents 4 0 R /Resources <</Font <</F1 5 0 R>>>>>>\nendobj"
        )
        .into_bytes(),
        &format!(
            "4 0 obj\n<</Length {stream_len}>>\nstream\n{}\nendstream\nendobj",
            std::str::from_utf8(stream_content).unwrap()
        )
        .into_bytes(),
        b"5 0 obj\n<</Type /Font /Subtype /Type1 /BaseFont /Helvetica>>\nendobj",
    ];

    let mut body = b"%PDF-1.4\n".to_vec();
    let mut offsets: Vec<usize> = Vec::new();
    for obj in objects {
        offsets.push(body.len());
        body.extend_from_slice(obj);
        body.push(b'\n');
    }
    let xref_offset = body.len();
    body.extend_from_slice(b"xref\n");
    let n = objects.len() + 1;
    body.extend_from_slice(format!("0 {}\n", n).as_bytes());
    body.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        body.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
    }
    body.extend_from_slice(
        format!("trailer\n<</Size {n} /Root 1 0 R>>\nstartxref\n{xref_offset}\n%%EOF\n").as_bytes(),
    );
    body
}

fn make_mp4() -> Vec<u8> {
    use ndarray::Array3;
    use std::path::Path;
    use video_rs::encode::{Encoder, Settings};
    use video_rs::time::Time;

    video_rs::init().expect("failed to init video-rs");

    let tmp = std::env::temp_dir().join("omni_fixture.mp4");
    let settings = Settings::preset_h264_yuv420p(320, 240, false);
    let mut encoder =
        Encoder::new(Path::new(&tmp), settings).expect("failed to create video encoder");

    // 1-second black video at 1 fps (1 frame)
    let frame: Array3<u8> = Array3::zeros((240, 320, 3));
    encoder
        .encode(&frame, Time::zero())
        .expect("failed to encode frame");
    encoder.finish().expect("failed to finish encoding");

    fs::read(&tmp).expect("failed to read encoded mp4")
}

fn b64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let n = match chunk.len() {
            3 => (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | chunk[2] as u32,
            2 => (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8,
            _ => (chunk[0] as u32) << 16,
        };
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
