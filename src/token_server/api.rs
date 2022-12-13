use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonObject, Value as JsonValue};

pub type MetaData = JsonObject<String, JsonValue>;
pub type Guid = String;

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
pub struct TokenOnlyPayload {
    pub token: Guid,
}

#[derive(Serialize)]
pub struct UpdateResponsePayload {
    pub token: Guid,
    pub meta: MetaData,
}

#[derive(Serialize)]
pub struct ValidateResponsePayload {
    pub meta: MetaData,
}
