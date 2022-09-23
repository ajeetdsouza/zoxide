use crate::cmd::{Edit, Run};
use crate::db::{Database, DatabaseFile, Epoch, Rank};
use crate::util::resolve_path;
use crate::{config, util};
use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

use dialoguer::Input;
use edit::edit;

const HEADER: &str = "\
# Blank lines and lines prepended with '#' are ignored; Line order is insignificant
# last_accessed,rank,path
";

enum ValidationResult {
    Success,
    Retry,
    Exit,
}

struct Problem {
    line_number: usize,
    text: String,
}

struct Problems {
    warnings: Vec<Problem>,
    errors: Vec<Problem>,
}

impl Problem {
    fn new(line_number: usize, text: String) -> Self {
        Problem { line_number, text }
    }
}

impl Run for Edit {
    fn run(&self) -> Result<()> {
        while let Some(db_edits) = get_db_edits()? {
            let data_dir = config::data_dir()?;
            let mut db_file = DatabaseFile::new(data_dir);
            let mut db = db_file.open()?;
            db.clear();
            let problems = get_problems(&mut db, db_edits);
            let result = handle_problems(problems);
            match result {
                ValidationResult::Success => {
                    db.save()?;
                    return Ok(());
                }
                ValidationResult::Exit => break,
                ValidationResult::Retry => continue,
            }
        }
        println!("Zoxide database not altered");
        Ok(())
    }
}

fn get_db_edits() -> Result<Option<String>> {
    let data_dir = config::data_dir()?;
    let mut db = DatabaseFile::new(data_dir);
    let mut db = db.open()?;
    let mut stream = db.stream(util::current_time().unwrap());
    let mut to_edit = String::from(HEADER);
    while let Some(dir) = stream.next() {
        writeln!(&mut to_edit, "{},{},{}", dir.last_accessed, dir.rank, dir.path)?;
    }
    let edit_result = edit(&to_edit)?;
    if edit_result == to_edit {
        // The file was not changed at all so we don't want to attempt to overwrite the original
        Ok(None)
    } else {
        Ok(Some(edit_result))
    }
}

fn get_problems(db: &mut Database, db_edits: String) -> Problems {
    let lines = db_edits.lines();

    let mut errors: Vec<Problem> = Vec::new();
    let mut warnings: Vec<Problem> = Vec::new();

    for (index, line) in lines.enumerate() {
        let line_number = index + 1;
        let first_char = line.trim().chars().next();
        if let Some(first_char) = first_char {
            if first_char == '#' {
                continue;
            }
        } else {
            continue;
        }
        let mut split = line.split(',');
        let (last_accessed_txt, rank_txt, path_txt) = (split.next(), split.next(), split.next());
        if split.next().is_some() {
            errors.push(Problem::new(line_number, "too many values on line".to_string()));
            continue;
        }

        let last_accessed: Option<Epoch> = match last_accessed_txt {
            Some(value) => match value.trim().parse::<Epoch>() {
                Ok(value) => Some(value),
                Err(e) => {
                    errors.push(Problem::new(line_number, e.to_string()));
                    None
                }
            },
            None => {
                errors.push(Problem::new(line_number, "Cannot parse 'last_accessed' field".to_string()));
                None
            }
        };

        let rank: Option<Rank> = match rank_txt {
            Some(value) => match value.trim().parse::<Rank>() {
                Ok(value) => Some(value),
                Err(e) => {
                    errors.push(Problem::new(line_number, e.to_string()));
                    None
                }
            },
            None => {
                errors.push(Problem::new(line_number, "Cannot parse 'rank' field".to_string()));
                None
            }
        };

        let path: Option<String> = match path_txt {
            Some(value) => {
                if value.trim() != value {
                    warnings.push(Problem::new(line_number, "path contains trailing whitespace".to_string()));
                }
                match resolve_path(&PathBuf::from(value)) {
                    Ok(v) => {
                        if v.to_str().unwrap() != value {
                            errors.push(Problem::new(line_number, "path must be an absolute path".to_string()));
                        }
                        Some(value.to_string())
                    }
                    Err(e) => {
                        errors.push(Problem::new(line_number, e.to_string()));
                        None
                    }
                }
            }
            None => {
                errors.push(Problem::new(line_number, "cannot parse 'path' field".to_string()));
                None
            }
        };

        if let (Some(path), Some(last_accessed), Some(rank)) = (path, last_accessed, rank) {
            db.add(&path, last_accessed, rank);
        }
    }
    Problems { warnings, errors }
}

fn handle_problems(problems: Problems) -> ValidationResult {
    let warnings = problems.warnings;
    let errors = problems.errors;
    let has_warnings = !warnings.is_empty();
    let has_errors = !errors.is_empty();
    if has_warnings {
        println!("Warnings:");
        for problem in warnings {
            println!("{}: {}", problem.line_number, problem.text);
        }
        println!();
    }
    if has_errors {
        println!("Errors:");
        for problem in errors {
            println!("line {}: {}", problem.line_number, problem.text);
        }
        println!();
    }
    if has_warnings || has_errors {
        println!("You may:");
        println!("(e)dit the file again");
        println!("e(x)it without saving changes");
        if !has_errors {
            println!("(s)ave changes and exit (DANGER!)");
        }
        let selection = Input::new()
            .with_prompt("Choice")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input == "e" || input == "x" || (input == "s" && !has_errors) {
                    Ok(())
                } else {
                    Err("Invalid selection.")
                }
            })
            .interact()
            .unwrap();
        return match selection.as_str() {
            "e" => ValidationResult::Retry,
            "s" => ValidationResult::Success,
            "x" => ValidationResult::Exit,
            i => panic!("Expected 'e', 's', or 'x'. Received {i}"), // We already validated input above
        };
    }
    ValidationResult::Success
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::db::DatabaseFile;

    use super::get_problems;

    fn maybe_translate_to_windows(line: String) -> String {
        if cfg!(windows) && line.contains("/tmp") {
            line.replace("/tmp", r"C:\tmp")
        } else {
            line
        }
    }

    #[test]
    fn no_validtion_problems() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db_file = DatabaseFile::new(temp_dir.path());
        let mut db = db_file.open().unwrap();
        let problems = get_problems(&mut db, maybe_translate_to_windows("1,1,/tmp".to_string()));
        assert!(problems.errors.is_empty());
        assert!(problems.warnings.is_empty());
    }

    #[rstest]
    #[case::invalid_accessed("1z,1,/tmp", "invalid digit found in string")]
    #[case::negative_accessed("-1,1,/tmp", "invalid digit found in string")]
    #[case::invalid_path("1,1,cool", "path must be an absolute path")]
    #[case::invalid_rank("1,1z,/tmp", "invalid float literal")]
    #[case::too_many_fields("1,1,1,/tmp", "too many values on line")]
    #[case::too_few_fields("1,1", "cannot parse 'path' field")]
    #[case::relative_path("1,1,~", "path must be an absolute path")]
    fn validation_error(#[case] line: String, #[case] err_text: &str) {
        let invalid_line = maybe_translate_to_windows(line);
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db_file = DatabaseFile::new(temp_dir.path());
        let mut db = db_file.open().unwrap();
        let problems = get_problems(&mut db, invalid_line);
        assert_eq!(problems.errors.len(), 1);
        assert!(problems.warnings.is_empty());
        assert_eq!(problems.errors[0].line_number, 1);
        assert_eq!(problems.errors[0].text, err_text);
    }

    #[test]
    fn validation_warning() {
        let invalid_line = maybe_translate_to_windows("1,1,/tmp ".to_string());
        let temp_dir = tempfile::tempdir().unwrap();
        let mut db_file = DatabaseFile::new(temp_dir.path());
        let mut db = db_file.open().unwrap();
        let problems = get_problems(&mut db, invalid_line);
        assert_eq!(problems.warnings.len(), 1);
        assert!(problems.errors.is_empty());
        assert_eq!(problems.warnings[0].line_number, 1);
        assert_eq!(problems.warnings[0].text, "path contains trailing whitespace");
    }
}
