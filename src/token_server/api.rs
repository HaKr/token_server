use std::{collections::HashMap, fmt::Display, time::Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub type MetaData = JsonValue;
pub type Guid = String;
pub type TokenStore = HashMap<Guid, (Instant, MetaData)>;

#[derive(Deserialize)]
pub struct CreatePayload {
    pub meta: MetaData,
}

#[derive(Deserialize)]
pub struct UpdatePayload {
    pub token: Guid,
    pub meta: Option<MetaData>,
}

#[derive(Deserialize)]
pub struct RemovePayload {
    pub token: Guid,
}

#[derive(Serialize)]
pub struct UpdateResponsePayload {
    pub token: Guid,
    pub meta: MetaData,
}

pub struct PurgeResult {
    pub tokens: usize,
    pub purged: usize,
}

impl Display for PurgeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "tokens: {}, purged: {}",
            self.tokens, self.purged
        ))
    }
}
