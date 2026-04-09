//! Pure formatting and small config-summary parsers for the GUI (testable on any host).

use surrogate_contract::events::Event;

pub fn parse_outbound_ids_from_summary(summary: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in summary.lines() {
        let prefix = "Outbounds: ";
        if let Some(rest) = line.strip_prefix(prefix) {
            for part in rest.split(',') {
                let s = part.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
            }
            break;
        }
    }
    out
}

pub fn event_detail_line(e: &Event) -> String {
    let kind = format!("{:?}", e.kind);
    let mut detail = e.message.clone();
    if let Some(h) = &e.host {
        detail.push_str(&format!(" — {h}"));
        if let Some(p) = e.port {
            detail.push_str(&format!(":{p}"));
        }
    }
    format!("session {} | {} | {}", e.session_id, kind, detail)
}

#[cfg(test)]
mod tests {
    use super::{event_detail_line, parse_outbound_ids_from_summary};
    use surrogate_contract::events::{Event, EventKind};

    #[test]
    fn parse_outbounds_splits_comma_separated_ids() {
        let summary = "Default Outbound: direct\nOutbounds: direct, reject\nRules: 0";
        let ids = parse_outbound_ids_from_summary(summary);
        assert_eq!(ids, vec!["direct".to_string(), "reject".to_string()]);
    }

    #[test]
    fn event_detail_line_contains_session_kind_message() {
        let e = Event {
            kind: EventKind::SessionStarted,
            session_id: 7,
            message: "session started".to_string(),
            host: Some("example.com".to_string()),
            port: Some(443),
            rule_id: None,
            outbound: None,
            bytes_from_client: None,
            bytes_from_upstream: None,
        };
        let s = event_detail_line(&e);
        assert!(s.contains("7"));
        assert!(s.contains("SessionStarted"));
        assert!(s.contains("example.com"));
    }
}
