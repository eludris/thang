pub type ThreadResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
