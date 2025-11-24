use std::env::var;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let in_docker = var("IN_DOCKER").is_ok();

    let commit_hash = {
        if Path::new("./.git").exists() {
            let output = Command::new("git")
                .arg("rev-parse")
                .arg("HEAD")
                .output()
                .expect("build.rs: Failed to execute command");

            if output.status.success() {
                let commit_hash_str = String::from_utf8_lossy(&output.stdout);
                commit_hash_str.trim().to_string()
            } else {
                println!("cargo::warning=build.rs: Git command failed with output: {output:#?}");
                String::from("unknown")
            }
        } else if !in_docker {
            println!("cargo::warning=build.rs: No .git directory found, skipping git versioning");
            String::from("unknown")
        } else {
            "".to_string()
        }
    };

    let env_suffix = match var("ACTIONS_ID") {
        Ok(id) => format!("(actions/runs/{id})"),
        Err(std::env::VarError::NotPresent) if !in_docker => "(Local)".to_string(),
        Err(_) if in_docker => "(Docker)".to_string(),
        Err(_) => "(unknown)".to_string(),
    };

    println!(
        "cargo:rustc-env=VERSION={}({} Profile)-{commit_hash}{env_suffix}[f:{}]",
        var("CARGO_PKG_VERSION").unwrap(),
        var("PROFILE").unwrap(),
        {
            let f = var("CARGO_CFG_FEATURE").unwrap();
            if f.is_empty() { "none".to_string() } else { f }
        }
    );
}
