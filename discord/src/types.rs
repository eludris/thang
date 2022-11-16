use std::error::Error;

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
