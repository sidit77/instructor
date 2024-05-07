#[derive(Debug)]
pub enum Error {
    TooShort,
    TooLong,
    InvalidValue,
    UnexpectedLength,
}