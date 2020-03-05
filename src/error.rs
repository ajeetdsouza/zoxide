use failure::Fail;

#[derive(Debug, Fail)]
pub enum AppError {
    #[fail(display = "found invalid UTF-8 code sequence")]
    UnicodeError,

    #[fail(display = "system clock is set to invalid time")]
    SystemTimeError,

    #[fail(display = "unable to open database file")]
    FileOpenError,
    #[fail(display = "unable to lock database file")]
    FileLockError,

    #[fail(display = "could not read from database")]
    DBReadError,
    #[fail(display = "could not write to database")]
    DBWriteError,

    #[fail(display = "could not launch fzf")]
    FzfLaunchError,
    #[fail(display = "could not communicate with fzf")]
    FzfIoError,

    #[fail(display = "could not retrieve home directory")]
    GetHomeDirError,
    #[fail(display = "could not retrieve current directory")]
    GetCurrentDirError,
    #[fail(display = "could not access path")]
    PathAccessError,
}
