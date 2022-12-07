use std::{collections::HashMap, sync::RwLock, time::Instant};

use chrono::{DateTime, Utc};
use duration_human::DurationHuman;
use serde::Serialize;
use tracing::debug;
use uuid::Uuid;

use super::{
    api::{Guid, MetaData, UpdateResponsePayload},
    purging::PurgeResult,
    RwLockNotAcquired, TokenUpdateFailed,
};

pub struct TokenServerState {
    tokens: RwLock<TokenStore>,
    started_at_instant: Instant,
    started_at_utc: DateTime<Utc>,
    token_lifetime: DurationHuman,
}

type TokenStore = HashMap<Guid, (Instant, MetaData)>;

#[derive(Serialize)]
struct DumpEntry<'de> {
    expires: String,
    meta: &'de MetaData,
}

impl Default for TokenServerState {
    fn default() -> Self {
        Self {
            tokens: RwLock::default(),
            started_at_instant: Instant::now(),
            started_at_utc: chrono::Utc::now(),
            token_lifetime: DurationHuman::default(),
        }
    }
}

impl TokenServerState {
    pub const fn with_token_lifetime(mut self, lifetime: DurationHuman) -> Self {
        self.token_lifetime = lifetime;

        self
    }

    pub fn create_token(&self, metadata: MetaData) -> Result<String, RwLockNotAcquired> {
        let mut tokens = self.tokens.write()?;

        let (tokenkey, expires) = self.new_token();
        let token = tokenkey.clone();

        tokens.insert(tokenkey, (expires, metadata));

        Ok(token)
    }

    pub fn update_token(
        &self,
        tokenkey: &String,
        metadata_update: Option<MetaData>,
    ) -> Result<UpdateResponsePayload, TokenUpdateFailed> {
        let mut tokens = self.tokens.write().map_err(RwLockNotAcquired::from)?;
        let now = Instant::now();

        let mut metadata = tokens.remove(tokenkey).map_or(
            Err(TokenUpdateFailed::InvalidToken),
            |(expires, metadata)| {
                if expires > now {
                    Ok(metadata)
                } else {
                    Err(TokenUpdateFailed::InvalidToken)
                }
            },
        )?;

        if let Some(metadata_update) = metadata_update {
            if let Some(existing) = metadata.as_object_mut() {
                if let Some(map) = metadata_update.as_object() {
                    for (k, v) in map {
                        existing.insert(k.to_string(), v.clone());
                    }
                }
            }
        }

        let (tokenkey, expires) = self.new_token();

        let meta = metadata.clone();
        tokens.insert(tokenkey.clone(), (expires, metadata));

        Ok(UpdateResponsePayload {
            token: tokenkey,
            meta,
        })
    }

    pub fn remove_token(&self, token: &String) -> Result<(), RwLockNotAcquired> {
        let mut tokens = self.tokens.write()?;

        let _meta = tokens.remove(token);

        Ok(())
    }

    pub fn remove_expired_tokens(&self) -> Result<PurgeResult, RwLockNotAcquired> {
        let mut tokens = self.tokens.write()?;
        let now = Instant::now();

        let tokens_before = tokens.len();
        tokens.retain(|_, (expires, _)| *expires >= now);

        let tokens = tokens.len();

        Ok(PurgeResult {
            tokens,
            purged: tokens_before - tokens,
        })
    }

    pub fn dump_meta(&self) {
        if let Ok(tokens) = self.tokens.read() {
            let report = tokens
                .iter()
                .map(|(_, (expires, meta))| {
                    let duration = expires.duration_since(self.started_at_instant);

                    // let's assume no wrap occurs, otherwise funny debug log
                    #[allow(clippy::cast_possible_wrap)]
                    let expires: DateTime<Utc> =
                        self.started_at_utc + chrono::Duration::seconds(duration.as_secs() as i64);

                    DumpEntry {
                        expires: expires.format("%Y-%m-%d %H:%M:%S").to_string(),
                        meta,
                    }
                })
                .collect::<Vec<DumpEntry>>();

            if let Ok(report) = serde_json::to_string(&report) {
                debug!("DUMP: {}", report);
            }
        }
    }
}

impl TokenServerState {
    #[inline]
    fn new_token(&self) -> (String, Instant) {
        (
            Uuid::new_v4().to_string(),
            self.token_lifetime + Instant::now(),
        )
    }
}
