use diesel_async::pooled_connection::deadpool;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Diesel: {0}")]
    Diesel(#[from] diesel::result::Error),

    #[error("Connection: {0}")]
    Connection(String),

    #[error("Database pool: {0}")]
    Pool(#[from] deadpool::PoolError),
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Diesel(diesel::result::Error::NotFound))
    }
}
