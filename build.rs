use std::env;
use std::process::Command;

macro_rules! warn {
    ($fmt:tt) => ({
        ::std::println!(::std::concat!("cargo:warning=", $fmt));
    });
    ($fmt:tt, $($arg:tt)*) => ({
        ::std::println!(::std::concat!("cargo:warning=", $fmt), $($arg)*);
    });
}

fn git_version() -> Option<String> {
    let mut git = Command::new("git");
    git.args(&["describe", "--tags", "--broken"]);

    let output = match git.output() {
        Err(e) => {
            warn!("when retrieving version: git failed to start: {}", e);
            return None;
        }
        Ok(output) if !output.status.success() => {
            warn!(
                "when retrieving version: git exited with code: {:?}",
                output.status.code()
            );
            return None;
        }
        Ok(output) => output,
    };

    match String::from_utf8(output.stdout) {
        Ok(version) => Some(version),
        Err(e) => {
            warn!("when retrieving version: git returned invalid utf-8: {}", e);
            None
        }
    }
}

fn crate_version() -> String {
    warn!("falling back to crate version");
    // unwrap is safe here, since Cargo will always supply this variable.
    format!("v{}", env::var("CARGO_PKG_VERSION").unwrap())
}

fn main() {
    let version = git_version().unwrap_or_else(crate_version);
    println!("cargo:rustc-env=ZOXIDE_VERSION={}", version);
}
