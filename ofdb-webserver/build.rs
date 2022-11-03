use std::{env, process::Command};

const CLEARANCE_APP_DIR: &str = "../ofdb-app-clearance";
const CLEARANCE_APP_SRC: &str = "../ofdb-app-clearance/src";
const CLEARANCE_FEATURE_NAME: &str = "clearance";

fn main() {
    if env::var(format!(
        "CARGO_FEATURE_{}",
        CLEARANCE_FEATURE_NAME.to_uppercase()
    ))
    .is_ok()
    {
        assert_trunk_is_installed();
        Command::new("trunk")
            .args(["build", "--release"])
            .current_dir(CLEARANCE_APP_DIR)
            .status()
            .expect("Unable to successfully execute trunk");
        println!("cargo:rerun-if-changed=\"{}\"", CLEARANCE_APP_SRC);
        for entry in walkdir::WalkDir::new(CLEARANCE_APP_SRC)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            println!("cargo:rerun-if-changed=\"{}\"", entry.path().display());
        }
    }
}

fn assert_trunk_is_installed() {
    let output = Command::new("cargo")
        .args(["install", "--list"])
        .output()
        .expect("Unable to check trunk installation");
    let output_string = String::from_utf8(output.stdout).unwrap();
    if !output_string.contains("trunk") {
        Command::new("cargo")
            .args(["install", "trunk"])
            .status()
            .expect("Unable install trunk");
    }
}
