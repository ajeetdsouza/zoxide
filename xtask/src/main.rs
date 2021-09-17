use std::process::Command;

use clap::Clap;

#[derive(Clap, Debug)]
struct App {
    #[clap(subcommand)]
    task: Task,
    #[clap(long)]
    nix: Option<bool>,
}

#[derive(Clap, Debug)]
enum Task {
    CI,
    Test { keywords: Vec<String> },
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
    let nix = app.nix.unwrap_or_else(|| Command::new("nix-shell").arg("--version").output().is_ok());
    let run = |args: &[&str]| run(args, nix);
    match app.task {
        Task::CI => {
            let color = if std::env::var_os("CI").is_some() { "--color=always" } else { "" };
            run(&["cargo", "fmt", "--", "--check", color, "--files-with-diff"]);
            run(&["cargo", "check", "--all-features", color]);
            run(&["cargo", "clippy", "--all-features", color, "--", "--deny=clippy::all", "--deny=warnings"]);
            run(&["cargo", "test", if nix { "--all-features" } else { "" }, color, "--no-fail-fast"]);
            run(&["cargo", "audit", color, "--deny=warnings"]);
            if nix {
                run(&["markdownlint", "--ignore-path=.gitignore", "."]);
            }
        }
        Task::Test { keywords } => {
            let mut args = vec!["cargo", "test", if nix { "--all-features" } else { "" }, "--no-fail-fast", "--"];
            args.extend(keywords.iter().map(String::as_str));
            run(&args);
        }
    }
}
