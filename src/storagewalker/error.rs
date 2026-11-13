use crate::ErrorNM;

#[derive(Debug)]
pub enum ErrorSW {
    ErrorNM(ErrorNM),
    UpHead,
    UnknownDir,
    InvalidObject,
}

impl From<ErrorNM> for ErrorSW {
    fn from(value: ErrorNM) -> Self {
        ErrorSW::ErrorNM(value)
    }
}
