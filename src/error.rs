//! Error types for jj-starship

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "jj")]
    #[error("jj: {0}")]
    Jj(String),

    #[cfg(feature = "git")]
    #[error("git: {0}")]
    Git(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
