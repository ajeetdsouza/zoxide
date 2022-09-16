use std::fmt::{self, Display, Formatter};
use std::io;

use anyhow::{bail, Context, Result};

/// Custom error type for early exit.
#[derive(Debug)]
pub struct SilentExit {
    pub code: u8,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub trait BrokenPipeHandler {
    fn pipe_exit(self, device: &str) -> Result<()>;
}

impl BrokenPipeHandler for io::Result<()> {
    fn pipe_exit(self, device: &str) -> Result<()> {
        match self {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => bail!(SilentExit { code: 0 }),
            result => result.with_context(|| format!("could not write to {device}")),
        }
    }
}
