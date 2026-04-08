// Ref: VLESS protocol specification (XTLS)

use super::{CleanRoomRisk, ConnectResult, ProtocolConfig, ProtocolError, ProtocolHandler};

pub struct VlessHandler;

impl ProtocolHandler for VlessHandler {
    fn name(&self) -> &str {
        "vless"
    }

    fn spec_reference(&self) -> &str {
        "VLESS protocol specification (XTLS)"
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
