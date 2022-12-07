// Looks like the Serialize on the TokenError causes this lint warning
// As an attribute on the enum itself doesn't help, I guess due to
// macro expansion, thus suppress it for this entire file
#![allow(clippy::use_self)]

use std::{fmt::Display, sync::PoisonError};

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum TokenError {
    #[error("InvalidToken")]
    InvalidToken,

    #[error("InternalServerError")]
    InternalServerError {
        #[from]
        source: RwLockNotAcquired,
    },

    #[error("Deserialize failed")]
    MustNeverOccur,
}

#[derive(Debug, Error, Serialize)]
pub struct RwLockNotAcquired;

impl Display for RwLockNotAcquired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("could not acquire read/write lock")
    }
}

// thiserror::from cannot accept anonymous lifetime specifier
impl<P> From<PoisonError<P>> for RwLockNotAcquired {
    fn from(_: PoisonError<P>) -> Self {
        Self {}
    }
}

// serde_josn::Error is not Serializable, as required by the #[from] attribute
impl From<serde_json::Error> for TokenError {
    fn from(_: serde_json::Error) -> Self {
        Self::MustNeverOccur
    }
}
