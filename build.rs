fn main() {
    let mut hash = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|proc| String::from_utf8(proc.stdout).ok())
        .unwrap_or_default();

    if !hash.is_empty() {
        hash = format!("-{}", hash);
    }

    println!("cargo:rustc-env=GIT_HASH={}", hash);
}
