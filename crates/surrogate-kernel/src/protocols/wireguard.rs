// Ref: WireGuard whitepaper (Jason Donenfeld, 2017)

use super::{CleanRoomRisk, ConnectResult, ProtocolConfig, ProtocolError, ProtocolHandler};

pub struct WireguardHandler;

impl ProtocolHandler for WireguardHandler {
    fn name(&self) -> &str {
        "wireguard"
    }

    fn spec_reference(&self) -> &str {
        "WireGuard whitepaper (Jason Donenfeld, 2017)"
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
