use std::{env, path::Path, process::Command};

const CLEARANCE_NAME: &str = "clearance";
const CLEARANCE_PKG_DIR: &str = "ofdb-app-clearance";
const CLEARANCE_PKG_SRC: &str = "ofdb-app-clearance/src";
const CLEARANCE_FEATURE_NAME: &str = "clearance";

fn main() {
    if env::var(format!(
        "CARGO_FEATURE_{}",
        CLEARANCE_FEATURE_NAME.to_uppercase()
    ))
    .is_ok()
    {
        assert_wasm_pack_is_installed();
        Command::new("wasm-pack")
            .args(&[
                "build",
                "--target",
                "web",
                "--release",
                "--out-name",
                CLEARANCE_NAME,
            ])
            .current_dir(&Path::new(&CLEARANCE_PKG_DIR))
            .status()
            .expect("Unable to successfully execute wasm-pack");
        println!("cargo:rerun-if-changed=\"{}\"", CLEARANCE_PKG_SRC);
        for entry in walkdir::WalkDir::new(CLEARANCE_PKG_SRC)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            println!("cargo:rerun-if-changed=\"{}\"", entry.path().display());
        }
    }
}

fn assert_wasm_pack_is_installed() {
    let output = Command::new("cargo")
        .args(&["install", "--list"])
        .output()
        .expect("Unable to check wasm-pack installation");
    let output_string = String::from_utf8(output.stdout).unwrap();
    if !output_string.contains("wasm-pack") {
        Command::new("cargo")
            .args(&["install", "wasm-pack"])
            .status()
            .expect("Unable install wasm-pack");
    }
}
