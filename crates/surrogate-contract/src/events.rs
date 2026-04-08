use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Trait for components that consume structured observability events.
pub trait EventSink: Send + Sync {
    /// Deliver a single event to this sink.
    fn emit(&self, event: Event);
}

/// Handle through which the proxy emits structured observability events.
#[derive(Clone)]
pub struct Observability {
    sink: Arc<dyn EventSink>,
}

impl Observability {
    /// Create an `Observability` instance backed by the given sink.
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self { sink }
    }

    /// Create a paired `(Observability, EventCollector)` for testing.
    pub fn collector() -> (Self, EventCollector) {
        let collector = EventCollector::default();
        let observability = Self::new(Arc::new(collector.clone()));
        (observability, collector)
    }

    /// Create an instance that writes JSON lines to stdout.
    pub fn stdout() -> Self {
        Self::new(Arc::new(StdoutJsonSink))
    }

    /// Emit a structured event through the configured sink.
    pub fn emit(&self, event: Event) {
        self.sink.emit(event);
    }
}

/// Discriminant for the kind of observability event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// A new proxy session was opened.
    SessionStarted,
    /// A routing rule matched the request.
    RuleMatched,
    /// The upstream connection was established.
    ForwardConnected,
    /// The upstream connection was closed.
    ForwardClosed,
    /// An error occurred during processing.
    Error,
}

/// Structured observability event emitted during proxy operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    /// What kind of event this is.
    pub kind: EventKind,
    /// Unique identifier for the proxy session.
    pub session_id: u64,
    /// Human-readable summary of the event.
    pub message: String,
    /// Target hostname, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Target port, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Identifier of the matched rule, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    /// Selected outbound transport, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbound: Option<String>,
    /// Bytes received from the downstream client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_from_client: Option<u64>,
    /// Bytes received from the upstream server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_from_upstream: Option<u64>,
}

impl Event {
    /// Create a `SessionStarted` event.
    pub fn session_started(session_id: u64, host: &str, port: u16) -> Self {
        Self {
            kind: EventKind::SessionStarted,
            session_id,
            message: "session started".to_string(),
            host: Some(host.to_string()),
            port: Some(port),
            rule_id: None,
            outbound: None,
            bytes_from_client: None,
            bytes_from_upstream: None,
        }
    }

    /// Create a `RuleMatched` event.
    pub fn rule_matched(
        session_id: u64,
        host: &str,
        port: u16,
        rule_id: &str,
        outbound: &str,
    ) -> Self {
        Self {
            kind: EventKind::RuleMatched,
            session_id,
            message: "rule matched".to_string(),
            host: Some(host.to_string()),
            port: Some(port),
            rule_id: Some(rule_id.to_string()),
            outbound: Some(outbound.to_string()),
            bytes_from_client: None,
            bytes_from_upstream: None,
        }
    }

    /// Create a `ForwardConnected` event.
    pub fn forward_connected(session_id: u64, host: &str, port: u16, outbound: &str) -> Self {
        Self {
            kind: EventKind::ForwardConnected,
            session_id,
            message: "forward connected".to_string(),
            host: Some(host.to_string()),
            port: Some(port),
            rule_id: None,
            outbound: Some(outbound.to_string()),
            bytes_from_client: None,
            bytes_from_upstream: None,
        }
    }

    /// Create a `ForwardClosed` event with byte counters.
    pub fn forward_closed(
        session_id: u64,
        bytes_from_client: u64,
        bytes_from_upstream: u64,
    ) -> Self {
        Self {
            kind: EventKind::ForwardClosed,
            session_id,
            message: "forward closed".to_string(),
            host: None,
            port: None,
            rule_id: None,
            outbound: None,
            bytes_from_client: Some(bytes_from_client),
            bytes_from_upstream: Some(bytes_from_upstream),
        }
    }

    /// Create an `Error` event.
    pub fn error(session_id: u64, message: impl Into<String>) -> Self {
        Self {
            kind: EventKind::Error,
            session_id,
            message: message.into(),
            host: None,
            port: None,
            rule_id: None,
            outbound: None,
            bytes_from_client: None,
            bytes_from_upstream: None,
        }
    }
}

/// In-memory event collector for use in tests.
#[derive(Clone, Default)]
pub struct EventCollector {
    events: Arc<Mutex<Vec<Event>>>,
}

impl EventCollector {
    /// Return a snapshot of all collected events.
    pub fn events(&self) -> Vec<Event> {
        self.events
            .lock()
            .expect("event collector lock should not be poisoned")
            .clone()
    }
}

impl EventSink for EventCollector {
    fn emit(&self, event: Event) {
        self.events
            .lock()
            .expect("event collector lock should not be poisoned")
            .push(event);
    }
}

/// Event sink that writes each event as a JSON line to stdout.
#[derive(Debug, Default)]
pub struct StdoutJsonSink;

impl EventSink for StdoutJsonSink {
    fn emit(&self, event: Event) {
        match serde_json::to_string(&event) {
            Ok(line) => {
                println!("{line}");
            }
            Err(error) => {
                eprintln!("failed to serialize observability event: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collector_records_structured_events() {
        let (observability, collector) = Observability::collector();
        observability.emit(Event::session_started(1, "example.com", 443));
        observability.emit(Event::rule_matched(
            1,
            "example.com",
            443,
            "default",
            "direct",
        ));

        let events = collector.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, EventKind::SessionStarted);
        assert_eq!(events[1].rule_id.as_deref(), Some("default"));
    }

    #[test]
    fn event_serde_roundtrip() {
        let event = Event::rule_matched(42, "example.com", 443, "block-ads", "reject");
        let json = serde_json::to_string(&event).expect("serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(event, deserialized);
    }

    #[test]
    fn event_factory_methods() {
        let started = Event::session_started(1, "host.com", 80);
        assert_eq!(started.kind, EventKind::SessionStarted);

        let matched = Event::rule_matched(2, "host.com", 80, "r1", "direct");
        assert_eq!(matched.kind, EventKind::RuleMatched);

        let connected = Event::forward_connected(3, "host.com", 80, "direct");
        assert_eq!(connected.kind, EventKind::ForwardConnected);

        let closed = Event::forward_closed(4, 100, 200);
        assert_eq!(closed.kind, EventKind::ForwardClosed);

        let error = Event::error(5, "something went wrong");
        assert_eq!(error.kind, EventKind::Error);
    }

    #[test]
    fn forward_closed_has_byte_counts() {
        let event = Event::forward_closed(10, 1024, 2048);
        assert_eq!(event.bytes_from_client, Some(1024));
        assert_eq!(event.bytes_from_upstream, Some(2048));
    }

    #[test]
    fn error_event_has_message() {
        let event = Event::error(99, "connection reset by peer");
        assert_eq!(event.message, "connection reset by peer");
        assert_eq!(event.session_id, 99);
    }
}
