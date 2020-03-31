fn main() {
    let git_describe = std::process::Command::new("git")
        .args(&["describe", "--tags", "--broken"])
        .output()
        .ok()
        .and_then(|proc| String::from_utf8(proc.stdout).ok());

    let mut version_info = format!("v{}-unknown", env!("CARGO_PKG_VERSION"));

    if let Some(description) = git_describe {
        if !description.is_empty() {
            version_info = description;
        }
    }

    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version_info);
}
