use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct SilentExit {
    pub code: i32,
}

impl Display for SilentExit {
    fn fmt(&self, _: &mut Formatter) -> fmt::Result {
        Ok(())
    }
}
