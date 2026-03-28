use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest_dir.parent().unwrap();

    let wcs = [
        ("omni-rt/crates/omni-dock", "omni-dock.js"),
        ("omni-rt/crates/omni-popper", "omni-popper.js"),
    ];

    for (crate_rel, js_name) in &wcs {
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
    }
}
