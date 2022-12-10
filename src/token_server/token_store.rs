use std::{collections::HashMap, sync::RwLock, time::Instant};

use chrono::{DateTime, Utc};
use duration_human::DurationHuman;

use tracing::debug;
use uuid::Uuid;

use super::{
    api::{Guid, MetaData, UpdateResponsePayload},
    formatting::{DumpEntry, PurgeResult},
    RwLockNotAcquired, TokenCreateFailed, TokenUpdateFailed,
};

pub struct TokenStore {
    tokens: RwLock<TokensByID>,
    started_at_instant: Instant,
    started_at_utc: DateTime<Utc>,
    token_lifetime: DurationHuman,
}

type TokensByID = HashMap<Guid, (Instant, MetaData)>;

impl TokenStore {
    pub const fn with_token_lifetime(mut self, lifetime: DurationHuman) -> Self {
        self.token_lifetime = lifetime;

        self
    }

    pub fn create_token(&self, metadata: MetaData) -> Result<String, TokenCreateFailed> {
        if !metadata.is_object() {
            return Err(TokenCreateFailed::MetaDataMustBeJsonObject);
        }

        self.tokens
            .write()
            .or(Err(TokenCreateFailed::RwLockNotAcquired))
            .map(|mut tokens| {
                let (token, expires) = self.new_token();

                tokens.insert(token.clone(), (expires, metadata));

                token
            })
    }

    pub fn remove_token(&self, token: &String) -> Result<(), RwLockNotAcquired> {
        self.tokens
            .write()
            .or(Err(RwLockNotAcquired))
            .map(|mut tokens| {
                tokens.remove(token);
            })
    }

    pub fn update_token(
        &self,
        tokenkey: &String,
        metadata_update: Option<MetaData>,
    ) -> Result<UpdateResponsePayload, TokenUpdateFailed> {
        self.tokens
            .write()
            .or(Err(TokenUpdateFailed::RwLockNotAcquired))
            .and_then(|mut tokens| {
                tokens
                    .remove(tokenkey)
                    .and_then(|(expires, mut meta)| {
                        if expires > Instant::now() {
                            let (token, expires) = self.new_token();

                            metadata_update.and_then(|metadata_update| {
                                metadata_update.as_object().and_then(
                                    |metadata_update_key_value_pairs| {
                                        meta.as_object_mut().map(|meta_key_value_pairs| {
                                            meta_key_value_pairs.extend(
                                                metadata_update_key_value_pairs
                                                    .iter()
                                                    .map(|(k, v)| (k.to_string(), v.clone())),
                                            );
                                        })
                                    },
                                )
                            });

                            tokens.insert(token.clone(), (expires, meta.clone()));
                            Some(UpdateResponsePayload { token, meta })
                        } else {
                            None
                        }
                    })
                    .ok_or(TokenUpdateFailed::InvalidToken)
            })
    }

    pub fn remove_expired_tokens(&self) -> Result<PurgeResult, RwLockNotAcquired> {
        self.tokens
            .write()
            .or(Err(RwLockNotAcquired))
            .map(|mut tokens| {
                let now = Instant::now();

                let tokens_before = tokens.len();
                tokens.retain(|_key, (expires, _meta)| *expires >= now);

                let tokens = tokens.len();

                PurgeResult {
                    tokens,
                    purged: tokens_before - tokens,
                }
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
                    DumpEntry::new(
                        self.started_at_utc + chrono::Duration::seconds(duration.as_secs() as i64),
                        meta,
                    )
                })
                .collect::<Vec<DumpEntry>>();

            if let Ok(report) = serde_json::to_string(&report) {
                debug!("DUMP: {}", report);
            }
        }
    }
}

impl TokenStore {
    #[inline]
    fn new_token(&self) -> (String, Instant) {
        (
            Uuid::new_v4().to_string(),
            self.token_lifetime + Instant::now(),
        )
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self {
            tokens: RwLock::default(),
            started_at_instant: Instant::now(),
            started_at_utc: chrono::Utc::now(),
            token_lifetime: DurationHuman::default(),
        }
    }
}
