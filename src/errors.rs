
use failure::Fail;
use std::path::{Path, PathBuf};
use std::clone::Clone;
use std::fmt::*;
use std::ops::Deref;

#[derive(Fail, Debug, Clone)]
pub(crate) enum AppError {
    #[fail(display = "Unable to stow {} to {} cause : {}", source, target, cause)]
    StowPathError {
        source: ErrorPath,
        target: ErrorPath,
        cause: String
    },

    #[fail(display = "An IO error append : {}", msg)]
    IOError {
        msg: String
    },

    #[fail(display = "Unable to apply stow because of previous errors")]
    ApplyError
}

#[derive(Debug, Clone)]
pub struct ErrorPath { path: PathBuf }

impl Deref for ErrorPath {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.path
    }
}

impl Display for ErrorPath {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.display())
    }
}

impl From<PathBuf> for ErrorPath {
    fn from(path: PathBuf) -> ErrorPath {
        ErrorPath { path }
    }
}

impl<'a> From<&'a Path> for ErrorPath {
    fn from(path: &Path) -> Self {
        ErrorPath { path: path.to_path_buf() }
    }
}