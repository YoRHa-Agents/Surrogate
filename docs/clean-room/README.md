# P-gate-0: Clean-Room / MIT Route Closure

**Status**: CLOSED  
**Date**: 2026-04-08  
**Decision**: D01 — Immediately freeze clean-room / MIT route

## Summary

The Surrogate project follows a **Rust-first, MIT-licensed, clean-room implementation** strategy.
All protocol modules must be implemented from public specifications, packet captures, interop
tests, or third-party writeups — never from reference implementation source code.

## Constraints

1. **License**: All dependencies must be MIT or MIT/Apache-2.0 dual-licensed. No GPL or copyleft.
2. **Clean-room**: Protocol implementations are derived solely from public documentation.
3. **Evidence**: Each protocol module must maintain an evidence log in this directory
   (`docs/clean-room/{protocol}-evidence.md`).
4. **Allowed sources**: `public-doc`, `packet-capture`, `interop-test`, `third-party-writeup`.
5. **Prohibited sources**: `reference-impl-source` (reading other implementations' source code).

## Protocol Evidence Logs

| Protocol | Evidence File | Status |
|----------|--------------|--------|
| Shadowsocks | `shadowsocks-evidence.md` | Pending (Stage E) |
| VLESS | `vless-evidence.md` | Pending (Stage E) |
| Trojan | `trojan-evidence.md` | Pending (Stage E) |
| VMess | `vmess-evidence.md` | Pending (Stage E, experimental) |
| WireGuard | `wireguard-evidence.md` | Pending (Stage E) |

## VMess Special Handling

VMess is classified as `clean-room-risk=HIGH` because no independent specification exists
outside the V2Ray source code. Implementation requires:
- VLESS must pass its gate first
- Evidence log coverage >= 80%
- Project lead sign-off before implementation begins
- Permanent `experimental` status until an independent spec is published
