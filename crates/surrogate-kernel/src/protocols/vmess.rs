// Ref: V2Ray VMess protocol (no independent spec — clean-room-risk=HIGH)

use super::{CleanRoomRisk, ConnectResult, ProtocolConfig, ProtocolError, ProtocolHandler};

pub struct VmessHandler;

impl ProtocolHandler for VmessHandler {
    fn name(&self) -> &str {
        "vmess"
    }

    fn spec_reference(&self) -> &str {
        "V2Ray VMess protocol (no independent spec — clean-room-risk=HIGH)"
    }

    fn clean_room_risk(&self) -> CleanRoomRisk {
        CleanRoomRisk::High
    }

    fn is_experimental(&self) -> bool {
        true
    }

    fn connect(&self, _target: &str, _config: &ProtocolConfig) -> ConnectResult<'_> {
        Box::pin(async {
            Err(ProtocolError::ConnectionFailed(
                "not yet implemented".into(),
            ))
        })
    }
}
