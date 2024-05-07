use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    TooShort,
    TooLong,
    InvalidValue,
    UnexpectedLength,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TooShort => write!(f, "Packet too short"),
            Error::TooLong => write!(f, "Packet too long"),
            Error::InvalidValue => write!(f, "Invalid value"),
            Error::UnexpectedLength => write!(f, "Unexpected length"),
        }
    }
}

impl std::error::Error for Error {}
