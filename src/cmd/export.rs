use std::fs;
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
                wtr.write_record(["path", "rank", "last_accessed"])
                    .context("could not write CSV header")?;
                for dir in dirs {
                    wtr.write_record([&*dir.path, &dir.rank.to_string(), &dir.last_accessed.to_string()])
                        .context("could not write CSV record")?;
                }
                wtr.flush().context("could not flush CSV writer")?;
                String::from_utf8(wtr.into_inner().context("could not get CSV bytes")?)
                    .context("CSV output is not valid UTF-8")?
            }
        };

        write_to_file(&self.out, &output)
            .with_context(|| format!("could not write to file: {}", self.out.display()))?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Dir;

    const TEST_ENTRIES: [(&str, f64, u64); 3] = [
        ("/home/alice/projects/zoxide", 42.5, 1714000000),
        ("/home/alice/downloads", 7.0, 1713000000),
        (r#"/tmp"quotes,commas""#, 1.0, 1712000000),
    ];

    fn create_test_db() -> tempfile::TempDir {
        let data_dir = tempfile::tempdir().unwrap();
        let mut db = Database::open_dir(data_dir.path()).unwrap();
        for (path, rank, last_accessed) in TEST_ENTRIES {
            db.add_unchecked(path, rank, last_accessed);
        }
        db.save().unwrap();
        data_dir
    }

    fn set_data_dir_env(data_dir: &tempfile::TempDir) {
        unsafe {
            std::env::set_var("_ZO_DATA_DIR", data_dir.path());
        }
    }

    fn find_entry_by_path<'a>(entries: &'a [Dir], path: &str) -> &'a Dir<'a> {
        entries.iter().find(|d| d.path == path).unwrap()
    }

    fn find_record_by_path<'a>(records: &'a [csv::StringRecord], path: &str) -> &'a csv::StringRecord {
        records.iter().find(|r| r.get(0).unwrap() == path).unwrap()
    }

    #[test]
    fn export_json() {
        let data_dir = create_test_db();
        set_data_dir_env(&data_dir);

        let out_file = data_dir.path().join("export.json");
        let export = Export {
            format: ExportFormat::Json,
            out: out_file.clone(),
        };
        export.run().unwrap();

        let content = fs::read_to_string(&out_file).unwrap();
        let result: Vec<Dir> = serde_json::from_str(&content).unwrap();

        assert_eq!(result.len(), 3);

        let entry1 = find_entry_by_path(&result, "/home/alice/projects/zoxide");
        assert!((entry1.rank - 42.5).abs() < 0.001);
        assert_eq!(entry1.last_accessed, 1714000000);

        let entry2 = find_entry_by_path(&result, "/home/alice/downloads");
        assert!((entry2.rank - 7.0).abs() < 0.001);
        assert_eq!(entry2.last_accessed, 1713000000);

        let entry3 = find_entry_by_path(&result, r#"/tmp"quotes,commas""#);
        assert!((entry3.rank - 1.0).abs() < 0.001);
        assert_eq!(entry3.last_accessed, 1712000000);
    }

    #[test]
    fn export_csv() {
        let data_dir = create_test_db();
        set_data_dir_env(&data_dir);

        let out_file = data_dir.path().join("export.csv");
        let export = Export {
            format: ExportFormat::Csv,
            out: out_file.clone(),
        };
        export.run().unwrap();

        let content = fs::read_to_string(&out_file).unwrap();
        let mut rdr = csv::Reader::from_reader(content.as_bytes());

        let headers = rdr.headers().unwrap();
        assert_eq!(headers, ["path", "rank", "last_accessed"].as_slice());

        let records: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 3);

        let record1 = find_record_by_path(&records, "/home/alice/projects/zoxide");
        assert_eq!(record1.get(1).unwrap().parse::<f64>().unwrap(), 42.5);
        assert_eq!(record1.get(2).unwrap().parse::<u64>().unwrap(), 1714000000);

        let record2 = find_record_by_path(&records, "/home/alice/downloads");
        assert_eq!(record2.get(1).unwrap().parse::<f64>().unwrap(), 7.0);
        assert_eq!(record2.get(2).unwrap().parse::<u64>().unwrap(), 1713000000);

        let record3 = find_record_by_path(&records, r#"/tmp"quotes,commas""#);
        assert_eq!(record3.get(1).unwrap().parse::<f64>().unwrap(), 1.0);
        assert_eq!(record3.get(2).unwrap().parse::<u64>().unwrap(), 1712000000);
    }

    #[test]
    fn export_csv_with_special_chars() {
        let data_dir = create_test_db();
        set_data_dir_env(&data_dir);

        let out_file = data_dir.path().join("export.csv");
        let export = Export {
            format: ExportFormat::Csv,
            out: out_file.clone(),
        };
        export.run().unwrap();

        let content = fs::read_to_string(&out_file).unwrap();
        assert!(content.contains(r#""""#));
        assert!(content.contains(r#"/tmp""#));

        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        let records: Vec<csv::StringRecord> = rdr.records().map(|r| r.unwrap()).collect();

        let special_record = records.iter().find(|r| r.get(0).unwrap().contains("quotes"));
        assert!(special_record.is_some());
        assert_eq!(special_record.unwrap().get(0).unwrap(), r#"/tmp"quotes,commas""#);
    }

    #[test]
    fn export_creates_parent_directories() {
        let data_dir = create_test_db();
        set_data_dir_env(&data_dir);

        let out_file = data_dir.path().join("nested").join("path").join("export.json");
        let export = Export {
            format: ExportFormat::Json,
            out: out_file.clone(),
        };
        export.run().unwrap();

        assert!(out_file.exists());
    }
}
