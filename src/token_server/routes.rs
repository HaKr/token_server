#![allow(clippy::unused_async)]
use std::sync::Arc;

use axum::{
    extract::{self, State},
    Json,
};
use http::StatusCode;

use super::{
    api::{CreatePayload, RemovePayload, UpdatePayload, UpdateResponsePayload},
    TokenError, TokenServerState,
};

pub async fn create_token(
    extract::State(state): State<Arc<TokenServerState>>,
    extract::Json(metadata): extract::Json<CreatePayload>,
) -> (StatusCode, String) {
    state.create_token(metadata.meta).map_or_else(
        |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)),
        |token| (StatusCode::OK, token),
    )
}

pub async fn update_token(
    State(state): State<Arc<TokenServerState>>,
    extract::Json(payload): extract::Json<UpdatePayload>,
) -> Json<Result<UpdateResponsePayload, TokenError>> {
    Json(state.update_token(&payload.token, payload.meta))
}

pub async fn remove_token(
    State(state): State<Arc<TokenServerState>>,
    extract::Json(payload): extract::Json<RemovePayload>,
) -> StatusCode {
    let _ = state.remove_token(&payload.token);

    StatusCode::ACCEPTED
}

pub async fn dump_meta(State(state): State<Arc<TokenServerState>>) -> StatusCode {
    let _ = state.dump_meta();

    StatusCode::ACCEPTED
}
