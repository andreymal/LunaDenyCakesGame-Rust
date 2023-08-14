use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SdlError(pub String);

impl Display for SdlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for SdlError {}
