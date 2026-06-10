use std::process::Command;

fn main() {
    // Bake the short git commit into the binary. Empty string when the
    // build happens outside a git checkout (release tarball, etc.).
    let commit = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    println!("cargo:rustc-env=PDFIX_GIT_COMMIT={commit}");

    // Trigger a rebuild whenever HEAD or the checked-out branch tip
    // changes — keeps the baked-in hash in sync with reality.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/heads");
    println!("cargo:rerun-if-changed=../.git/packed-refs");

    tauri_build::build();
}
