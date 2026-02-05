use std::borrow::Cow;
use std::io::{self, Write};
use std::path::{self, Path};

use anyhow::{Context, Result};

use crate::cmd::{Query, Run};
use crate::config;
use crate::db::{Database, Dir, Epoch, Stream, StreamOptions};
use crate::error::BrokenPipeHandler;
use crate::util::{self, Fzf, FzfChild};

impl Run for Query {
    fn run(&self) -> Result<()> {
        let mut db = crate::db::Database::open()?;
        self.query(&mut db).and(db.save())
    }
}

impl Query {
    fn query(&self, db: &mut Database) -> Result<()> {
        let now = util::current_time()?;
        let suffix_query = SuffixQuery::from_keywords(&self.keywords);

        if let Some(suffix_query) = suffix_query {
            self.query_with_suffix(db, now, suffix_query)
        } else {
            let mut stream = self.get_stream(db, now)?;
            if self.interactive {
                self.query_interactive(&mut stream, now)
            } else if self.list {
                self.query_list(&mut stream, now)
            } else {
                self.query_first(&mut stream, now)
            }
        }
    }

    fn query_interactive(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        let mut fzf = Self::get_fzf()?;
        let selection = loop {
            match stream.next() {
                Some(dir) if Some(dir.path.as_ref()) == self.exclude.as_deref() => continue,
                Some(dir) => {
                    if let Some(selection) = fzf.write(dir, now)? {
                        break selection;
                    }
                }
                None => break fzf.wait()?,
            }
        };

        if self.score {
            print!("{selection}");
        } else {
            let path = selection.get(7..).context("could not read selection from fzf")?;
            print!("{path}");
        }
        Ok(())
    }

    fn query_list(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        let handle = &mut io::stdout().lock();
        while let Some(dir) = stream.next() {
            if Some(dir.path.as_ref()) == self.exclude.as_deref() {
                continue;
            }
            let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
            writeln!(handle, "{dir}").pipe_exit("stdout")?;
        }
        Ok(())
    }

    fn query_first(&self, stream: &mut Stream, now: Epoch) -> Result<()> {
        let handle = &mut io::stdout();

        let mut dir = stream.next().context("no match found")?;
        while Some(dir.path.as_ref()) == self.exclude.as_deref() {
            dir = stream.next().context("you are already in the only match")?;
        }

        let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
        writeln!(handle, "{dir}").pipe_exit("stdout")
    }

    fn get_stream<'a>(&self, db: &'a mut Database, now: Epoch) -> Result<Stream<'a>> {
        self.get_stream_with_keywords(db, now, &self.keywords)
    }

    fn get_stream_with_keywords<'a>(
        &self,
        db: &'a mut Database,
        now: Epoch,
        keywords: &[String],
    ) -> Result<Stream<'a>> {
        let mut options = StreamOptions::new(now)
            .with_keywords(keywords.iter().map(|s| s.as_str()))
            .with_exclude(config::exclude_dirs()?)
            .with_base_dir(self.base_dir.clone());
        if !self.all {
            let resolve_symlinks = config::resolve_symlinks();
            options = options.with_exists(true).with_resolve_symlinks(resolve_symlinks);
        }

        let stream = Stream::new(db, options);
        Ok(stream)
    }

    fn get_fzf() -> Result<FzfChild> {
        let mut fzf = Fzf::new()?;
        if let Some(fzf_opts) = config::fzf_opts() {
            fzf.env("FZF_DEFAULT_OPTS", fzf_opts)
        } else {
            fzf.args([
                // Search mode
                "--exact",
                // Search result
                "--no-sort",
                // Interface
                "--bind=ctrl-z:ignore,btab:up,tab:down",
                "--cycle",
                "--keep-right",
                // Layout
                "--border=sharp", // rounded edges don't display correctly on some terminals
                "--height=45%",
                "--info=inline",
                "--layout=reverse",
                // Display
                "--tabstop=1",
                // Scripting
                "--exit-0",
            ])
            .enable_preview()
        }
        .spawn()
    }

    fn query_with_suffix(&self, db: &mut Database, now: Epoch, suffix_query: SuffixQuery) -> Result<()> {
        if self.interactive {
            if self.query_interactive_normal(db, now)? {
                Ok(())
            } else if self.query_interactive_suffix(db, now, &suffix_query)? {
                Ok(())
            } else {
                anyhow::bail!("no match found")
            }
        } else if self.list {
            if self.query_list_normal(db, now)? {
                Ok(())
            } else {
                self.query_list_suffix(db, now, &suffix_query)
            }
        } else {
            match self.query_first_normal(db, now)? {
                FirstMatch::Found => Ok(()),
                FirstMatch::OnlyExcluded => {
                    if self.query_first_suffix(db, now, &suffix_query)? {
                        Ok(())
                    } else {
                        anyhow::bail!("you are already in the only match")
                    }
                }
                FirstMatch::NoMatch => {
                    if self.query_first_suffix(db, now, &suffix_query)? {
                        Ok(())
                    } else {
                        anyhow::bail!("no match found")
                    }
                }
            }
        }
    }

    fn query_interactive_normal(&self, db: &mut Database, now: Epoch) -> Result<bool> {
        let mut stream = self.get_stream(db, now)?;
        let mut fzf = Self::get_fzf()?;
        let mut wrote_any = false;
        let mut selection = None;
        while let Some(dir) = stream.next() {
            if Some(dir.path.as_ref()) == self.exclude.as_deref() {
                continue;
            }
            wrote_any = true;
            if let Some(result) = fzf.write(dir, now)? {
                selection = Some(result);
                break;
            }
        }

        if !wrote_any {
            let _ = fzf.wait();
            return Ok(false);
        }

        let selection = match selection {
            Some(selection) => selection,
            None => fzf.wait()?,
        };

        if self.score {
            print!("{selection}");
        } else {
            let path = selection.get(7..).context("could not read selection from fzf")?;
            print!("{path}");
        }
        Ok(true)
    }

    fn query_interactive_suffix(
        &self,
        db: &mut Database,
        now: Epoch,
        suffix_query: &SuffixQuery,
    ) -> Result<bool> {
        let mut stream = self.get_stream_with_keywords(db, now, &suffix_query.base_keywords)?;
        let mut fzf = Self::get_fzf()?;
        let mut wrote_any = false;
        let mut selection = None;
        while let Some(dir) = stream.next() {
            if let Some(dir) = self.suffix_dir(dir, &suffix_query.suffix)? {
                wrote_any = true;
                if let Some(result) = fzf.write(&dir, now)? {
                    selection = Some(result);
                    break;
                }
            }
        }

        if !wrote_any {
            let _ = fzf.wait();
            return Ok(false);
        }

        let selection = match selection {
            Some(selection) => selection,
            None => fzf.wait()?,
        };

        if self.score {
            print!("{selection}");
        } else {
            let path = selection.get(7..).context("could not read selection from fzf")?;
            print!("{path}");
        }
        Ok(true)
    }

    fn query_list_normal(&self, db: &mut Database, now: Epoch) -> Result<bool> {
        let mut stream = self.get_stream(db, now)?;
        let handle = &mut io::stdout().lock();
        let mut wrote_any = false;
        while let Some(dir) = stream.next() {
            if Some(dir.path.as_ref()) == self.exclude.as_deref() {
                continue;
            }
            wrote_any = true;
            let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
            writeln!(handle, "{dir}").pipe_exit("stdout")?;
        }
        Ok(wrote_any)
    }

    fn query_list_suffix(
        &self,
        db: &mut Database,
        now: Epoch,
        suffix_query: &SuffixQuery,
    ) -> Result<()> {
        let mut stream = self.get_stream_with_keywords(db, now, &suffix_query.base_keywords)?;
        let handle = &mut io::stdout().lock();
        while let Some(dir) = stream.next() {
            if let Some(dir) = self.suffix_dir(dir, &suffix_query.suffix)? {
                let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
                writeln!(handle, "{dir}").pipe_exit("stdout")?;
            }
        }
        Ok(())
    }

    fn query_first_normal(&self, db: &mut Database, now: Epoch) -> Result<FirstMatch> {
        let mut stream = self.get_stream(db, now)?;
        let mut dir = match stream.next() {
            Some(dir) => dir,
            None => return Ok(FirstMatch::NoMatch),
        };
        while Some(dir.path.as_ref()) == self.exclude.as_deref() {
            dir = match stream.next() {
                Some(dir) => dir,
                None => return Ok(FirstMatch::OnlyExcluded),
            };
        }
        let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
        writeln!(&mut io::stdout(), "{dir}").pipe_exit("stdout")?;
        Ok(FirstMatch::Found)
    }

    fn query_first_suffix(
        &self,
        db: &mut Database,
        now: Epoch,
        suffix_query: &SuffixQuery,
    ) -> Result<bool> {
        let mut stream = self.get_stream_with_keywords(db, now, &suffix_query.base_keywords)?;
        let handle = &mut io::stdout();
        while let Some(dir) = stream.next() {
            if let Some(dir) = self.suffix_dir(dir, &suffix_query.suffix)? {
                let dir = if self.score { dir.display().with_score(now) } else { dir.display() };
                writeln!(handle, "{dir}").pipe_exit("stdout")?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn suffix_dir(&self, dir: &Dir<'_>, suffix: &str) -> Result<Option<Dir<'static>>> {
        let candidate = Path::new(dir.path.as_ref()).join(suffix);
        if !candidate.is_dir() {
            return Ok(None);
        }
        let candidate = util::path_to_str(&candidate)?;
        if Some(candidate) == self.exclude.as_deref() {
            return Ok(None);
        }
        Ok(Some(Dir {
            path: Cow::Owned(candidate.to_string()),
            rank: dir.rank,
            last_accessed: dir.last_accessed,
        }))
    }
}

#[derive(Debug)]
struct SuffixQuery {
    base_keywords: Vec<String>,
    suffix: String,
}

impl SuffixQuery {
    fn from_keywords(keywords: &[String]) -> Option<Self> {
        for (idx, keyword) in keywords.iter().enumerate() {
            if let Some((base, suffix)) = split_suffix(keyword) {
                if base.is_empty() || suffix.is_empty() {
                    continue;
                }
                let mut base_keywords = keywords.to_vec();
                base_keywords[idx] = base.to_string();
                return Some(Self { base_keywords, suffix: suffix.to_string() });
            }
        }
        None
    }
}

fn split_suffix(keyword: &str) -> Option<(&str, &str)> {
    for (idx, ch) in keyword.char_indices() {
        if path::is_separator(ch) {
            let base = &keyword[..idx];
            let suffix = &keyword[idx + ch.len_utf8()..];
            return Some((base, suffix));
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
enum FirstMatch {
    Found,
    NoMatch,
    OnlyExcluded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_suffix_parses_first_separator() {
        let result = SuffixQuery::from_keywords(&[String::from("abc/uniq_child")]).unwrap();
        assert_eq!(result.base_keywords, vec![String::from("abc")]);
        assert_eq!(result.suffix, "uniq_child");
    }

    #[test]
    fn split_suffix_ignores_empty_parts() {
        assert!(SuffixQuery::from_keywords(&[String::from("abc/")]).is_none());
        assert!(SuffixQuery::from_keywords(&[String::from("/uniq")]).is_none());
    }

    #[test]
    fn split_suffix_replaces_only_one_keyword() {
        let result =
            SuffixQuery::from_keywords(&[String::from("abc/uniq"), String::from("extra")])
                .unwrap();
        assert_eq!(result.base_keywords, vec![String::from("abc"), String::from("extra")]);
        assert_eq!(result.suffix, "uniq");
    }
}
