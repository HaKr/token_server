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
        |e| match e {
            TokenCreateFailed::MetaDataMustBeJsonObject(e) => {
                error!("{}", e);
                (StatusCode::OK, format!("ERROR: {}", e))
            }
            TokenCreateFailed::RwLockNotAcquired(e) => internal_server_error(e),
        },
        |token| (StatusCode::OK, token),
    )
}

pub async fn update_token(
    State(state): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<UpdatePayload>,
) -> Response {
    let res = state.update_token(&payload.token, payload.meta);

    match res {
        Err(TokenUpdateFailed::InternalServerError(err)) => {
            internal_server_error(err).into_response()
        }
        _ => Json(res).into_response(),
    }
}

pub async fn remove_token(
    State(state): State<Arc<TokenStore>>,
    extract::Json(payload): extract::Json<RemovePayload>,
) -> (StatusCode, String) {
    state
        .remove_token(&payload.token)
        .map_or_else(internal_server_error, |()| {
            (StatusCode::ACCEPTED, String::new())
        })
}

pub async fn dump_meta(State(state): State<Arc<TokenStore>>) -> StatusCode {
    state.dump_meta();

    StatusCode::ACCEPTED
}

#[inline]
fn internal_server_error(e: RwLockNotAcquired) -> (StatusCode, String) {
    let msg = format!("{}", e);
    error!(msg);
    (StatusCode::INTERNAL_SERVER_ERROR, msg)
}
