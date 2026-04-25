use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::cmd::{Export, ExportFormat, Run};
use crate::db::Database;

impl Run for Export {
    fn run(&self) -> Result<()> {
        let db = Database::open()?;
        let dirs = db.dirs();

        let output = match self.format {
            ExportFormat::Json => serde_json::to_string(dirs)
                .context("could not serialize to JSON")?,
            ExportFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(Vec::new());
                for dir in dirs {
                    wtr.write_record([&*dir.path, &dir.rank.to_string(), &dir.last_accessed.to_string()])
                        .context("could not write CSV record")?;
                }
                wtr.flush().context("could not flush CSV writer")?;
                String::from_utf8(wtr.into_inner().context("could not get CSV bytes")?)
                    .context("CSV output is not valid UTF-8")?
            }
        };

        match &self.out {
            Some(path) => {
                write_to_file(path, &output)
                    .with_context(|| format!("could not write to file: {}", path.display()))?;
            }
            None => {
                writeln!(io::stdout(), "{output}")
                    .context("could not write to stdout")?;
            }
        }

        Ok(())
    }
}

fn write_to_file(path: impl AsRef<Path>, content: &str) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("could not create directory: {}", parent.display()))?;
        }
    }
    fs::write(path, content)
        .with_context(|| format!("could not write to file: {}", path.display()))?;
    Ok(())
}
