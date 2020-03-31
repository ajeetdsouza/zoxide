fn main() {
    let git_describe = std::process::Command::new("git")
        .args(&["describe", "--tags", "--broken"])
        .output()
        .ok()
        .and_then(|proc| String::from_utf8(proc.stdout).ok());

    let version_info = match git_describe {
        Some(description) if !description.is_empty() => description,
        _ => format!("v{}-unknown", env!("CARGO_PKG_VERSION")),
    };

    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version_info);
}
