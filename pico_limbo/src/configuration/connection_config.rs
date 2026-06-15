use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionConfig {
    /// Interval between two `minecraft:keep_alive` packets sent to a client
    /// while in CONFIGURATION or PLAY state, in seconds.
    ///
    /// Vanilla uses 15. Lower it (e.g. 10) if a proxy in front of `PicoLimbo`
    /// has a stricter read-timeout. Has no effect on clients <= 1.7.6, which
    /// use a fixed 2-second ping required by the legacy protocol.
    pub keep_alive_interval_seconds: u64,

    /// If set to true, `PicoLimbo` will attempt to use the latest protocol
    /// version for unsupported versions. Useful for snapshots.
    pub allow_unsupported_versions: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            keep_alive_interval_seconds: 15,
            allow_unsupported_versions: false,
        }
    }
}
