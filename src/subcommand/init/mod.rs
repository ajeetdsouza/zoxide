mod shell;

use anyhow::Result;
use clap::arg_enum;
use structopt::StructOpt;

use std::io::{self, Write};

#[derive(Debug, StructOpt)]
#[structopt(about = "Generates shell configuration")]
pub struct Init {
    #[structopt(possible_values = &Shell::variants(), case_insensitive = true)]
    shell: Shell,

    #[structopt(
        long,
        help = "Changes the name of the 'z' command",
        default_value = "z"
    )]
    z_cmd: String,

    #[structopt(
        long,
        help = "Prevents zoxide from defining any commands other than 'z'"
    )]
    no_define_aliases: bool,

    #[structopt(
        long,
        help = "Chooses event on which an entry is added to the database",
        possible_values = &Hook::variants(),
        default_value = "pwd",
        case_insensitive = true
    )]
    hook: Hook,
}

impl Init {
    pub fn run(&self) -> Result<()> {
        let config = match self.shell {
            Shell::bash => shell::bash::CONFIG,
            Shell::fish => shell::fish::CONFIG,
            Shell::posix => shell::posix::CONFIG,
            Shell::zsh => shell::zsh::CONFIG,
        };

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        let z = config.z;
        writeln!(handle, "{}", z(&self.z_cmd)).unwrap();

        if !self.no_define_aliases {
            let alias = config.alias;
            writeln!(handle, "{}", alias(&self.z_cmd)).unwrap();
        }

        match self.hook {
            Hook::none => (),
            Hook::prompt => writeln!(handle, "{}", config.hook.prompt).unwrap(),
            Hook::pwd => {
                let hook_pwd = config.hook.pwd;
                writeln!(handle, "{}", hook_pwd()?).unwrap();
            }
        }

        Ok(())
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Shell {
        bash,
        fish,
        posix,
        zsh,
    }
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Hook {
        none,
        prompt,
        pwd,
    }
}
