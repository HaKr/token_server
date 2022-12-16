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
    RwLockNotAcquired, TokenStore, TokenUpdateFailed,
};

pub async fn create_token(
    extract::State(token_store): State<Arc<TokenStore>>,
    extract::Json(metadata): extract::Json<CreatePayload>,
) -> (StatusCode, String) {
    token_store.create_token(metadata.meta).map_or_else(
        |_err| {
            ResponseFromResult::internal_server_error()
                .log()
                .into_tuple()
        },
        |token| (StatusCode::OK, token),
    )
}

pub async fn update_token(
    State(token_store): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<UpdatePayload>,
) -> Response {
    let update_result = token_store.update_token(&payload.token, payload.meta);

    match update_result {
        Err(TokenUpdateFailed::RwLockNotAcquired) => ResponseFromResult::internal_server_error()
            .log()
            .into_response(),
        _ => Json(update_result).into_response(),
    }
}

pub async fn remove_token(
    State(token_store): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<RemovePayload>,
) -> Response {
    token_store.remove_token(&payload.token).map_or_else(
        |_e| {
            ResponseFromResult::internal_server_error()
                .log()
                .into_response()
        },
        |()| StatusCode::ACCEPTED.into_response(),
    )
}

pub async fn dump_meta(State(token_store): State<Arc<TokenStore>>) -> StatusCode {
    token_store.dump_meta();

    StatusCode::ACCEPTED
}

pub async fn shutdown_server(extract::State(token_store): State<Arc<TokenStore>>) -> StatusCode {
    token_store.shutdown();
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
            status_text: format!("{RwLockNotAcquired}",),
            log_message: format!("{RwLockNotAcquired:#}"),
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
