use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Result of a health probe check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    /// The component is operating normally.
    Healthy,
    /// The component is functional but exhibiting problems.
    Degraded(String),
    /// The component is non-functional.
    Unhealthy(String),
}

/// Async health-check probe that components implement to report liveness.
pub trait HealthProbe: Send + Sync {
    /// Human-readable name of this probe (e.g. `"dns-resolver"`).
    fn name(&self) -> &str;
    /// Execute the probe and return the current health status.
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;
    /// Maximum duration the probe runner should wait before treating the
    /// check as timed-out. Defaults to 5 seconds.
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProbe;

    impl HealthProbe for MockProbe {
        fn name(&self) -> &str {
            "mock-probe"
        }

        fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
            Box::pin(async { HealthStatus::Healthy })
        }
    }

    #[test]
    fn health_probe_default_timeout() {
        let probe = MockProbe;
        assert_eq!(probe.timeout(), Duration::from_secs(5));
    }

    #[test]
    fn health_status_serde_roundtrip() {
        let variants = vec![
            HealthStatus::Healthy,
            HealthStatus::Degraded("high latency".to_string()),
            HealthStatus::Unhealthy("connection refused".to_string()),
        ];
        for status in &variants {
            let json = serde_json::to_string(status).expect("serialize");
            let deserialized: HealthStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(status, &deserialized);
        }
    }

    #[tokio::test]
    async fn mock_health_probe_compiles() {
        let probe = MockProbe;
        assert_eq!(probe.name(), "mock-probe");
        let status = probe.check().await;
        assert_eq!(status, HealthStatus::Healthy);
    }
}
