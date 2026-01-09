use std::env::var;
use std::path::Path;
use std::process::Command;

fn warn(t: &'static str) {
    println!("cargo:warning=build.rs: {t}");
}

fn main() {
    // init
    for f in ["build.rs", "emojis.json"] {
        println!("cargo:rerun-if-changed={f}");
    }

    // env
    let in_docker = var("IN_DOCKER").is_ok();

    let commit_hash = {
        if Path::new("./.git").exists() {
            let output = Command::new("git").arg("rev-parse").arg("HEAD").output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        let commit_hash_str = String::from_utf8_lossy(&output.stdout);
                        commit_hash_str.trim().to_string()
                    } else {
                        warn("Git command failed with output: {output:#?}");
                        String::from("unknown")
                    }
                }
                Err(e) => {
                    println!(
                        "cargo:warning=build.rs: Failed to execute git command = {}",
                        e
                    );

                    String::from("unknown")
                }
            }
        } else if !in_docker {
            warn("No .git directory found, skipping git versioning");
            String::from("unknown")
        } else {
            "".to_string()
        }
    };

    let env_suffix = match var("ACTIONS_ID") {
        Ok(id) => format!("actions/runs/{id}"),
        Err(std::env::VarError::NotPresent) if !in_docker => "Local".to_string(),
        Err(_) if in_docker => "Docker".to_string(),
        Err(_) => "unknown".to_string(),
    };
    println!(
        "cargo:rustc-env=FEATURES={}",
        var("CARGO_CFG_FEATURE").unwrap_or_default()
    );
    println!(
        "cargo:rustc-env=PROFILE={}",
        var("PROFILE").unwrap_or("unknown".to_string())
    );
    println!("cargo:rustc-env=commit_hash={commit_hash}");
    println!("cargo:rustc-env=env_suffix={env_suffix}");
}
