pub trait ExitIfError<T, E> {
    fn exit_if_error(self) -> T;
}

impl<T, E: std::fmt::Display> ExitIfError<T, E> for Result<T, E> {
    fn exit_if_error(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}
