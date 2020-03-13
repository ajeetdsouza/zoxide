use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Env {
    #[serde(default = "default_maxage")]
    pub maxage: i64,

    #[serde(default = "default_data")]
    pub data: Option<PathBuf>,
}

fn default_maxage() -> i64 {
    1000
}

fn default_data() -> Option<PathBuf> {
    let mut path = dirs::home_dir()?;
    path.push(".zo");
    Some(path)
}
