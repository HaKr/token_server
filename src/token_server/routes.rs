#![allow(clippy::unused_async)]
use std::sync::Arc;

use axum::{extract, response::IntoResponse, Extension, Json};
use http::StatusCode;

use super::{
    api::{CreatePayload, RemovePayload, UpdatePayload, UpdateResponsePayload},
    TokenError, TokenServerState,
};

pub async fn create_token(
    Extension(state): Extension<Arc<TokenServerState>>,
    extract::Json(metadata): extract::Json<CreatePayload>,
) -> impl IntoResponse {
    match state.create_token(metadata.meta) {
        Ok(token) => (StatusCode::OK, token),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
    }
}

pub async fn update_token(
    Extension(state): Extension<Arc<TokenServerState>>,
    extract::Json(payload): extract::Json<UpdatePayload>,
) -> Json<Result<UpdateResponsePayload, TokenError>> {
    Json(state.update_token(&payload.token, payload.meta))
}

pub async fn remove_token(
    Extension(state): Extension<Arc<TokenServerState>>,
    extract::Json(payload): extract::Json<RemovePayload>,
) -> StatusCode {
    let _ = state.remove_token(&payload.token);

    StatusCode::ACCEPTED
}

pub async fn dump_meta(Extension(state): Extension<Arc<TokenServerState>>) -> StatusCode {
    let _ = state.dump_meta();

    StatusCode::ACCEPTED
}
