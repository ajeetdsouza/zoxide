use anyhow::{bail, Context, Result};

use std::fmt::{self, Display, Formatter};
use std::io;

// Custom error type for early exit.
#[derive(Debug)]
pub struct SilentExit {
    pub code: i32,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}

pub trait WriteErrorHandler {
    fn handle_err(self, device: &str) -> Result<()>;
}

impl WriteErrorHandler for io::Result<()> {
    fn handle_err(self, device: &str) -> Result<()> {
        match self {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => bail!(SilentExit { code: 0 }),
            result => result.with_context(|| format!("could not write to {}", device)),
        }
    }
}
