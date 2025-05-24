use anyhow::Result;

use crate::cmd::{Clear, Remove, Run};
use crate::db::Database;
use std::io;
impl Run for Clear {
    fn run(&self) -> Result<()> {
        let db = Database::open()?;

        if db.dirs().len() == 0 {
            println!("History already clear")
        }

        let mut path = Remove { paths: Vec::new() };

        for dir in db.dirs() {
            path.paths.push(dir.path.to_string());
        }

        println!("Do you want to remove {} elements from history? <Y/N>", path.paths.len());

        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.read_line(&mut buffer)?;

        match buffer.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                let _ = path.run()?;
            }

            "n" | "no" => {
                println!("Nothing was deleted")
            }
            _ => println!("Wrong option use Y, N, Yes, No"),
        };

        Ok(())
    }
}
