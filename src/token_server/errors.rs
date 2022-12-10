// Looks like the Serialize on the TokenError causes this lint warning
// As an attribute on the enum itself doesn't help, I guess due to
// macro expansion, thus suppress it for this entire file
#![allow(clippy::use_self)]

use std::fmt::Display;

use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum TokenCreateFailed {
    #[error("metadata must be a JSON object")]
    MetaDataMustBeJsonObject,

    #[error("InternalServerError")]
    RwLockNotAcquired,
}

#[derive(Error, Debug, Serialize)]
pub enum TokenUpdateFailed {
    #[error("InvalidToken")]
    InvalidToken,

    #[error("InternalServerError")]
    RwLockNotAcquired,

    #[error("Deserialize failed")]
    MustNeverOccur,
}

#[derive(Debug, Error, Serialize, Copy, Clone)]
pub struct RwLockNotAcquired;

impl Display for RwLockNotAcquired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if f.alternate() {
            "could not acquire read/write lock"
        } else {
            "InternalServerError"
        })
    }
}

// serde_josn::Error is not Serializable, as required by the #[from] attribute
impl From<serde_json::Error> for TokenUpdateFailed {
    fn from(_: serde_json::Error) -> Self {
        Self::MustNeverOccur
    }
}
