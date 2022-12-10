use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};

use crate::token_server::api::MetaData;

#[derive(Serialize)]
pub struct DumpEntry<'de> {
    #[serde(serialize_with = "format_expiration")]
    expires: DateTime<Utc>,
    meta: &'de MetaData,
}

impl<'de> DumpEntry<'de> {
    pub const fn new(expires: DateTime<Utc>, meta: &'de MetaData) -> Self {
        Self { expires, meta }
    }
}

fn format_expiration<S>(expires: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(expires.format("%Y-%m-%d %H:%M:%S").to_string().as_str())
}
