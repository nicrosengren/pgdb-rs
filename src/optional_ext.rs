use crate::Error;

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
