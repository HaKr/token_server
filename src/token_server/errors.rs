// Looks like the Serialize on the TokenError causes this lint warning
// As an attribute on the enum itself doesn't help, I guess due to
// macro expansion, thus suppress it for this entire file
#![allow(clippy::use_self)]

use std::sync::PoisonError;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum TokenError {
    #[error("InvalidToken")]
    InvalidToken,

    #[error("InternalServerError")]
    InternalServerError,

    #[error("Deserialize failed")]
    MustNeverOccur,
}

// thiserror::from cannot accept anonymous lifetime specifier
impl<P> From<PoisonError<P>> for TokenError {
    fn from(_: PoisonError<P>) -> Self {
        Self::InternalServerError
    }
}

// serde_josn::Error is not Serializable, as required by the #[from] attribute
impl From<serde_json::Error> for TokenError {
    fn from(_: serde_json::Error) -> Self {
        Self::MustNeverOccur
    }
}
