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
