use std::env;
use std::process::Command;

fn main() {
    // Get version from git tags
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    
    println!("cargo:rustc-env=VESKTOP_VOICE_OVERLAY_VERSION={}", version);
    
    // Rebuild if git changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
