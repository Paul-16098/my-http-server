use std::env::var;
use std::path::Path;
use std::process::Command;

fn main() {
  #[allow(clippy::single_element_loop)]
  for path in ["build.rs"] {
    println!("cargo:rerun-if-changed={path}");
  }

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
    } else {
      println!("cargo::warning=build.rs: No .git directory found, skipping git versioning");
      String::from("unknown")
    }
  };

  println!(
    "cargo:rustc-env=VERSION={}({} Profile)-{commit_hash}({})",
    var("CARGO_PKG_VERSION").unwrap(),
    var("PROFILE").unwrap(),
    match var("ACTIONS_ID") {
      Ok(id) => format!("actions/runs/{id}"),
      Err(std::env::VarError::NotPresent) => "Local".to_string(),
      Err(e) => {
        println!("cargo::error=build.rs: {e}");
        "unknown".to_string()
      }
    }
  );
}
