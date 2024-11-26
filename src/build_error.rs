use crate::Error;
use diesel_async::pooled_connection::deadpool;
use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("{0}")]
    Pool(#[from] deadpool::BuildError),

    #[error("testing connection: {0}")]
    Connection(#[from] Error),

    #[error("{0}")]
    Other(Cow<'static, str>),
}

impl BuildError {
    pub fn other(s: impl Into<Cow<'static, str>>) -> Self {
        Self::Other(s.into())
    }
}
