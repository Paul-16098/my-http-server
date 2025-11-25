use std::env::var;
use std::path::Path;
use std::process::Command;

fn warn(t: &'static str) {
    println!("cargo:warning=build.rs: {t}");
}

#[cfg(feature = "github_emojis")]
fn download_github_emojis() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("./emojis.json").exists() {
        warn("No emojis.json found, downloading...");
        // Download emojis.json using Rust (reqwest blocking)
        let url = "http://api.github.com/emojis";

        let mut resp = ureq::get(url)
            .header("User-Agent", "Paul-16098/my-http-server-build")
            .call()?;
        println!("{:?}", resp);
        println!("build.rs: Downloaded emojis.json, writing to file...");
        let body_str = resp.body_mut().read_to_string()?;
        std::fs::write("emojis.json", body_str)?;
    } else {
        println!("build.rs: emojis.json found, skipping download");
    }
    Ok(())
}

fn main() {
    // init
    for f in ["build.rs", "emojis.json"] {
        println!("cargo:rerun-if-changed={f}");
    }
    // download
    #[cfg(feature = "github_emojis")]
    if let Err(e) = download_github_emojis() {
        panic!("{}", e);
    };

    // env
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
                warn("Git command failed with output: {output:#?}");
                String::from("unknown")
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
        var("CARGO_CFG_FEATURE").unwrap()
    );
    println!("cargo:rustc-env=PROFILE={}", var("PROFILE").unwrap());
    println!("cargo:rustc-env=commit_hash={commit_hash}");
    println!("cargo:rustc-env=env_suffix={env_suffix}");
}
