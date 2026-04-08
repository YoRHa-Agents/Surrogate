// Ref: Trojan Protocol specification (trojan-gfw)

use super::{CleanRoomRisk, ConnectResult, ProtocolConfig, ProtocolError, ProtocolHandler};

pub struct TrojanHandler;

impl ProtocolHandler for TrojanHandler {
    fn name(&self) -> &str {
        "trojan"
    }

    fn spec_reference(&self) -> &str {
        "Trojan Protocol specification (trojan-gfw)"
    }

    fn clean_room_risk(&self) -> CleanRoomRisk {
        CleanRoomRisk::Low
    }

    fn connect(&self, _target: &str, _config: &ProtocolConfig) -> ConnectResult<'_> {
        Box::pin(async {
            Err(ProtocolError::ConnectionFailed(
                "not yet implemented".into(),
            ))
        })
    }
}
