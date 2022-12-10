#![allow(clippy::unused_async)]
use std::sync::Arc;

use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;

use tracing::error;

use super::{
    api::{CreatePayload, RemovePayload, UpdatePayload},
    RwLockNotAcquired, TokenCreateFailed, TokenStore, TokenUpdateFailed,
};

pub async fn create_token(
    extract::State(state): State<Arc<TokenStore>>,
    extract::Json(metadata): extract::Json<CreatePayload>,
) -> (StatusCode, String) {
    state.create_token(metadata.meta).map_or_else(
        |err| ResponseFromResult::from(err).log().into_tuple(),
        |token| (StatusCode::OK, token),
    )
}

pub async fn update_token(
    State(state): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<UpdatePayload>,
) -> Response {
    let update_result = state.update_token(&payload.token, payload.meta);

    match update_result {
        Err(TokenUpdateFailed::RwLockNotAcquired) => ResponseFromResult::internal_server_error()
            .log()
            .into_response(),
        _ => Json(update_result).into_response(),
    }
}

pub async fn remove_token(
    State(state): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<RemovePayload>,
) -> Response {
    state.remove_token(&payload.token).map_or_else(
        |_e| {
            ResponseFromResult::internal_server_error()
                .log()
                .into_response()
        },
        |()| StatusCode::ACCEPTED.into_response(),
    )
}

pub async fn dump_meta(State(state): State<Arc<TokenStore>>) -> StatusCode {
    state.dump_meta();

    StatusCode::ACCEPTED
}

struct ResponseFromResult {
    status_code: StatusCode,
    status_text: String,
    log_message: String,
}

impl ResponseFromResult {
    /// respond with 500 Internal server error and write error message to the server log
    fn internal_server_error() -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            status_text: format!("{}", RwLockNotAcquired),
            log_message: format!("{:#}", RwLockNotAcquired),
        }
    }

    /// write an error message to the server log
    fn log(self) -> Self {
        error!("{}", self.log_message);

        self
    }

    /// move the contents to a `(StatusCode, String)` tuple
    #[allow(clippy::missing_const_for_fn)]
    fn into_tuple(self) -> (StatusCode, String) {
        (self.status_code, self.status_text)
    }

    /// move the contents to a `http::Response`
    #[deny(clippy::missing_const_for_fn)]
    fn into_response(self) -> Response {
        self.into_tuple().into_response()
    }
}

impl From<TokenCreateFailed> for ResponseFromResult {
    fn from(err: TokenCreateFailed) -> Self {
        match err {
            TokenCreateFailed::MetaDataMustBeJsonObject => Self {
                status_code: StatusCode::UNSUPPORTED_MEDIA_TYPE,
                status_text: String::from("ERROR: metadata must be a JSON object"),

                log_message: String::from("Received invalid JSON data"),
            },
            TokenCreateFailed::RwLockNotAcquired => Self::internal_server_error(),
        }
    }
}
