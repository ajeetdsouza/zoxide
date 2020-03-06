use crate::error::AppError;
use crate::types::Rank;
use failure::{bail, ResultExt};
use std::env;
use std::ffi::OsString;

pub const ZO_DATA: &str = "_ZO_DATA";
pub const ZO_MAXAGE: &str = "_ZO_MAXAGE";

pub fn get_zo_data() -> Result<OsString, failure::Error> {
    let path = match env::var_os(ZO_DATA) {
        Some(path) => path,
        None => {
            let mut path = dirs::home_dir().ok_or_else(|| AppError::GetHomeDirError)?;
            path.push(".zo");
            path.into_os_string()
        }
    };
    Ok(path)
}

pub fn get_zo_maxage() -> Result<Rank, failure::Error> {
    if let Some(maxage_osstr) = env::var_os(ZO_MAXAGE) {
        match maxage_osstr.to_str() {
            Some(maxage_str) => {
                let maxage = maxage_str
                    .parse::<Rank>()
                    .with_context(|_| AppError::EnvError(ZO_MAXAGE.to_owned()))?;
                Ok(maxage)
            }
            None => bail!(AppError::EnvError(ZO_MAXAGE.to_owned())),
        }
    } else {
        Ok(5000.0)
    }
}
