// build.rs
use std::process::Command;

fn main() {
  // Use git command or a Rust crate like git2 to get the commit hash
  let output = Command::new("git")
    .args(["rev-parse", "HEAD"])
    .output()
    .expect("Failed to execute git command");

  let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
  println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash);
}
