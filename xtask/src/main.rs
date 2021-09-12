use clap::{ArgEnum, Clap};

use std::process::Command;

#[derive(Clap, Debug)]
struct App {
    #[clap(arg_enum)]
    task: Task,
    #[clap(long)]
    nix: Option<bool>,
}

#[derive(ArgEnum, Debug)]
enum Task {
    CI,
}

fn run(args: &[&str], nix: bool) {
    let args_str = args.join(" ");
    println!(">>> {}", args_str);

    let status = if nix {
        Command::new("nix-shell").args(&["--pure", "--run", &args_str]).status()
    } else {
        let (cmd, args) = args.split_first().unwrap();
        Command::new(cmd).args(args).status()
    };
    if !status.unwrap().success() {
        panic!("command exited with an error");
    }
}

fn main() {
    let app = App::parse();
    let color = if std::env::var_os("CI").is_some() { "--color=always" } else { "--color=auto" };
    let nix = app.nix.unwrap_or_else(|| Command::new("nix-shell").arg("--version").output().is_ok());
    let run = |args: &[&str]| run(args, nix);
    match app.task {
        Task::CI => {
            run(&["cargo", "fmt", "--", "--check", color, "--files-with-diff"]);
            run(&["cargo", "check", "--all-features", color]);
            run(&["cargo", "clippy", "--all-features", color, "--", "--deny=warnings", "--deny=clippy::all"]);
            run(&["cargo", "test", if nix { "--all-features" } else { "" }, color, "--no-fail-fast"]);
            // color: https://github.com/rustsec/rustsec/pull/436
            run(&["cargo", "audit", "--deny=warnings"]);
        }
    }
}
