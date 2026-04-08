use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingProtocol {
    Http2,
    WebSocket,
}

#[derive(Debug, Error)]
pub enum StreamingError {
    #[error("ALPN negotiation failed: {0}")]
    AlpnFailed(String),
    #[error("WebSocket upgrade failed: {0}")]
    WsUpgradeFailed(String),
    #[error("mid-stream break: {0}")]
    MidStreamBreak(String),
    #[error("TLS version mismatch: {0}")]
    TlsVersionMismatch(String),
    #[error("connection pool exhausted")]
    PoolExhausted,
    #[error("HTTP/2 stream concurrency limit exceeded")]
    H2ConcurrencyLimit,
    #[error("WebSocket ping/pong timeout")]
    WsPingTimeout,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackStrategy {
    DowngradeToHttp11,
    RetryThenSwitch { max_retries: u32 },
    TransparentReconnect { max_attempts: u32 },
    DowngradeTls12,
    QueueWait { max_queue: usize },
    ExpandPool,
    CloseAndReconnect,
}

#[derive(Debug)]
pub struct FailureMode {
    pub error_kind: &'static str,
    pub strategy: FallbackStrategy,
    pub description: &'static str,
}

pub fn default_failure_modes() -> Vec<FailureMode> {
    vec![
        FailureMode {
            error_kind: "AlpnFailed",
            strategy: FallbackStrategy::DowngradeToHttp11,
            description: "ALPN negotiation failed — downgrade to HTTP/1.1",
        },
        FailureMode {
            error_kind: "WsUpgradeFailed",
            strategy: FallbackStrategy::RetryThenSwitch { max_retries: 1 },
            description: "WebSocket upgrade failed — retry once then switch to HTTP/2",
        },
        FailureMode {
            error_kind: "MidStreamBreak",
            strategy: FallbackStrategy::TransparentReconnect { max_attempts: 3 },
            description: "Mid-stream break — transparent reconnect up to 3 times",
        },
        FailureMode {
            error_kind: "TlsVersionMismatch",
            strategy: FallbackStrategy::DowngradeTls12,
            description: "TLS version mismatch — downgrade to TLS 1.2",
        },
        FailureMode {
            error_kind: "PoolExhausted",
            strategy: FallbackStrategy::QueueWait { max_queue: 128 },
            description: "Connection pool exhausted — queue with bounded wait (max 128)",
        },
        FailureMode {
            error_kind: "H2ConcurrencyLimit",
            strategy: FallbackStrategy::ExpandPool,
            description: "HTTP/2 stream concurrency exceeded — expand connection pool",
        },
        FailureMode {
            error_kind: "WsPingTimeout",
            strategy: FallbackStrategy::CloseAndReconnect,
            description: "WebSocket ping/pong timeout — close and reconnect",
        },
    ]
}

pub struct StreamingMetrics {
    pub fallback_count: AtomicU64,
    pub reconnect_count: AtomicU64,
    pub pool_rejection_count: AtomicU64,
    pub total_streams: AtomicU64,
}

impl StreamingMetrics {
    pub fn new() -> Self {
        Self {
            fallback_count: AtomicU64::new(0),
            reconnect_count: AtomicU64::new(0),
            pool_rejection_count: AtomicU64::new(0),
            total_streams: AtomicU64::new(0),
        }
    }
}

impl Default for StreamingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StreamingLayer {
    metrics: Arc<StreamingMetrics>,
    failure_modes: Vec<FailureMode>,
}

impl StreamingLayer {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(StreamingMetrics::new()),
            failure_modes: default_failure_modes(),
        }
    }

    pub fn metrics(&self) -> &StreamingMetrics {
        &self.metrics
    }

    pub fn failure_modes(&self) -> &[FailureMode] {
        &self.failure_modes
    }

    pub async fn handle_upgrade_request(
        &self,
        _protocol: StreamingProtocol,
    ) -> Result<(), StreamingError> {
        self.metrics.total_streams.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn record_failure(&self, error: &StreamingError) {
        match error {
            StreamingError::AlpnFailed(_) | StreamingError::TlsVersionMismatch(_) => {
                self.metrics.fallback_count.fetch_add(1, Ordering::Relaxed);
            }
            StreamingError::MidStreamBreak(_)
            | StreamingError::WsPingTimeout
            | StreamingError::WsUpgradeFailed(_) => {
                self.metrics.reconnect_count.fetch_add(1, Ordering::Relaxed);
            }
            StreamingError::PoolExhausted | StreamingError::H2ConcurrencyLimit => {
                self.metrics
                    .pool_rejection_count
                    .fetch_add(1, Ordering::Relaxed);
            }
            StreamingError::Io(_) => {
                self.metrics.reconnect_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl Default for StreamingLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_failure_modes_has_seven_entries() {
        let modes = default_failure_modes();
        assert_eq!(modes.len(), 7);

        let kinds: Vec<&str> = modes.iter().map(|m| m.error_kind).collect();
        assert!(kinds.contains(&"AlpnFailed"));
        assert!(kinds.contains(&"WsUpgradeFailed"));
        assert!(kinds.contains(&"MidStreamBreak"));
        assert!(kinds.contains(&"TlsVersionMismatch"));
        assert!(kinds.contains(&"PoolExhausted"));
        assert!(kinds.contains(&"H2ConcurrencyLimit"));
        assert!(kinds.contains(&"WsPingTimeout"));
    }

    #[test]
    fn streaming_metrics_increment() {
        let metrics = StreamingMetrics::new();

        assert_eq!(metrics.fallback_count.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.reconnect_count.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.pool_rejection_count.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_streams.load(Ordering::Relaxed), 0);

        metrics.fallback_count.fetch_add(1, Ordering::Relaxed);
        metrics.reconnect_count.fetch_add(3, Ordering::Relaxed);
        metrics.pool_rejection_count.fetch_add(2, Ordering::Relaxed);
        metrics.total_streams.fetch_add(10, Ordering::Relaxed);

        assert_eq!(metrics.fallback_count.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.reconnect_count.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.pool_rejection_count.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_streams.load(Ordering::Relaxed), 10);
    }

    #[test]
    fn record_failure_updates_correct_counter() {
        let layer = StreamingLayer::new();

        layer.record_failure(&StreamingError::AlpnFailed("no h2".into()));
        layer.record_failure(&StreamingError::TlsVersionMismatch("1.3 required".into()));
        assert_eq!(layer.metrics().fallback_count.load(Ordering::Relaxed), 2);
        assert_eq!(layer.metrics().reconnect_count.load(Ordering::Relaxed), 0);
        assert_eq!(
            layer.metrics().pool_rejection_count.load(Ordering::Relaxed),
            0
        );

        layer.record_failure(&StreamingError::MidStreamBreak("reset".into()));
        layer.record_failure(&StreamingError::WsPingTimeout);
        layer.record_failure(&StreamingError::WsUpgradeFailed("401".into()));
        assert_eq!(layer.metrics().reconnect_count.load(Ordering::Relaxed), 3);

        layer.record_failure(&StreamingError::PoolExhausted);
        layer.record_failure(&StreamingError::H2ConcurrencyLimit);
        assert_eq!(
            layer.metrics().pool_rejection_count.load(Ordering::Relaxed),
            2
        );
    }

    #[test]
    fn streaming_error_display() {
        let err = StreamingError::AlpnFailed("no h2 offered".into());
        assert_eq!(err.to_string(), "ALPN negotiation failed: no h2 offered");

        let err = StreamingError::WsUpgradeFailed("403 forbidden".into());
        assert_eq!(err.to_string(), "WebSocket upgrade failed: 403 forbidden");

        let err = StreamingError::MidStreamBreak("connection reset".into());
        assert_eq!(err.to_string(), "mid-stream break: connection reset");

        let err = StreamingError::TlsVersionMismatch("server requires 1.3".into());
        assert_eq!(err.to_string(), "TLS version mismatch: server requires 1.3");

        let err = StreamingError::PoolExhausted;
        assert_eq!(err.to_string(), "connection pool exhausted");

        let err = StreamingError::H2ConcurrencyLimit;
        assert_eq!(err.to_string(), "HTTP/2 stream concurrency limit exceeded");

        let err = StreamingError::WsPingTimeout;
        assert_eq!(err.to_string(), "WebSocket ping/pong timeout");

        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let err = StreamingError::Io(io_err);
        assert_eq!(err.to_string(), "I/O error: broken pipe");
    }
}
