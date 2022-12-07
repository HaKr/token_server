use std::fmt::Display;

pub struct PurgeResult {
    pub tokens: usize,
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
