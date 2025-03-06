pub struct AnyError {
    message: String,
}

impl<T> From<T> for AnyError
where
    T: std::fmt::Display,
{
    fn from(e: T) -> Self {
        AnyError {
            message: e.to_string(),
        }
    }
}

impl ToString for AnyError {
    fn to_string(&self) -> String {
        self.message.clone()
    }
}

pub type AnyResult<T> = Result<T, AnyError>;
