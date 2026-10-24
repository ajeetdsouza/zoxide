#[path = "src/cmd/cmd.rs"]
mod cmd;

use std::path::Path;
use std::{env, fs, io};

use clap::CommandFactory as _;
use clap_complete::Generator;
use clap_complete::shells::{Bash, Elvish, Fish, PowerShell, Zsh};
use clap_complete_fig::Fig;
use clap_complete_nushell::Nushell;
use cmd::Cmd;

fn main() -> io::Result<()> {
    // Since we are generating completions in the package directory, we need to
    // set this so that Cargo doesn't rebuild every time.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=tests/");

    generate_completions(Bash)?;
    generate_completions(Elvish)?;
    generate_completions(Fig)?;
    generate_completions(Fish)?;
    generate_completions(Nushell)?;
    generate_completions(PowerShell)?;
    generate_completions(Zsh)?;

    Ok(())
}

/// Writes a shell's completions only if they differ from the committed file.
fn generate_completions(generator: impl Generator) -> io::Result<()> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");
    const OUT_DIR: &str = "contrib/completions";

    let path = Path::new(OUT_DIR).join(generator.file_name(BIN_NAME));
    let mut generated = Vec::new();
    clap_complete::generate(generator, &mut Cmd::command(), BIN_NAME, &mut generated);

    if !fs::read(&path).is_ok_and(|content| content == generated) {
        fs::write(&path, generated)?;
    }
    Ok(())
}
