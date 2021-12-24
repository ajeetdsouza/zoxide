use std::io;

use anyhow::{bail, Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("could not find fzf, is it installed?")]
pub struct FzfNotFound;

/// Custom error type for early exit.
#[derive(Debug, Error)]
#[error("")]
pub struct SilentExit {
    pub code: i32,
}

pub trait BrokenPipeHandler {
    fn pipe_exit(self, device: &str) -> Result<()>;
}

impl BrokenPipeHandler for io::Result<()> {
    fn pipe_exit(self, device: &str) -> Result<()> {
        match self {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => bail!(SilentExit { code: 0 }),
            result => result.with_context(|| format!("could not write to {}", device)),
        }
    }
}
