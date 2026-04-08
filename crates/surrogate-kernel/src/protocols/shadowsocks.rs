// Ref: Shadowsocks AEAD/2022 specification

use super::{CleanRoomRisk, ConnectResult, ProtocolConfig, ProtocolError, ProtocolHandler};

pub struct ShadowsocksHandler;

impl ProtocolHandler for ShadowsocksHandler {
    fn name(&self) -> &str {
        "shadowsocks"
    }

    fn spec_reference(&self) -> &str {
        "Shadowsocks AEAD/2022 specification"
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
