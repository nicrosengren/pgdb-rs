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

pub trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, Error>;
}

impl<T> OptionalExt<T> for Result<T, Error> {
    fn optional(self) -> Result<Option<T>, Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(Error::Diesel(diesel::result::Error::NotFound)) => Ok(None),
            Err(err) => {
                println!("was not not found: {err}");
                Err(err)
            }
        }
    }
}
