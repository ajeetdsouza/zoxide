use std::fmt::{self, Display};

#[derive(Debug)]
pub struct SilentExit {
    pub code: i32,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
