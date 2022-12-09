use std::fmt::Display;

pub struct PurgeResult {
    /// number of tokens left after purge
    pub tokens: usize,

    /// number of tokens removed during this purge
    pub purged: usize,
}

impl Display for PurgeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "PURGED: tokens: {}, purged: {}",
            self.tokens, self.purged
        ))
    }
}
