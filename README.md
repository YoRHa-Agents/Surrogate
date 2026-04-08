# Surrogate

A high-performance, cross-platform proxy kernel written in Rust. Surrogate provides a modular data plane with HTTP CONNECT and SOCKS5 support, a rule-based routing engine, and a pluggable control plane — all built with a clean-room, MIT-licensed approach.

## Features

- **Dual Proxy Protocol** — HTTP CONNECT tunneling and SOCKS5 (CONNECT command) with automatic protocol detection
- **Rule-Based Routing** — Indexed rule matching by exact host, host suffix, and port with configurable outbound selection (direct / reject)
- **Structured Observability** — Full session lifecycle events (`SessionStarted`, `RuleMatched`, `ForwardConnected`, `ForwardClosed`, `Error`) via a pluggable `EventSink` trait
- **Modular Architecture** — Five-crate workspace with strict dependency boundaries:
  - `surrogate-contract` — shared types, traits, and domain objects
  - `surrogate-kernel` — data plane (proxy engine, rule matching, relay)
  - `surrogate-control` — control plane (config store, rule compiler, plugin registry, import engine)
  - `surrogate-bridge` — platform-specific traffic capture (Linux explicit proxy, macOS stub)
  - `surrogate-app` — CLI entry point
- **Plugin System** — Capability-driven plugin registry with six built-in plugins
- **Config Import** — Neutral IR-based import engine supporting Clash, sing-box, and V2Ray config migration
- **Cross-Platform** — Linux (explicit proxy mode) and macOS (Network Extension path planned)
- **Clean-Room Protocols** — Protocol handler framework for Shadowsocks, VLESS, Trojan, VMess, and WireGuard (currently scaffolded; implementations in progress)

## Architecture

```
┌──────────────────────────────────────────────┐
│                surrogate-app                 │
│            (CLI + assembly layer)            │
├──────────────────┬───────────────────────────┤
│ surrogate-kernel │     surrogate-control     │
│   (data plane)   │     (control plane)       │
│                  │                           │
│  HTTP CONNECT    │  ConfigStore              │
│  SOCKS5          │  RuleCompiler             │
│  RuleRegistry    │  PluginRegistry           │
│  ConnectionPool  │  ImportEngine             │
│  StreamingLayer  │  TestWorkbench            │
│  Protocols       │  CoverageAnalysis         │
│                  │  AbilityLens              │
├──────────────────┴───────────────────────────┤
│             surrogate-contract               │
│   (domain types, traits, events, errors)     │
├──────────────────────────────────────────────┤
│             surrogate-bridge                 │
│  (Linux explicit proxy / macOS NE planned)   │
└──────────────────────────────────────────────┘
```

**Dependency Rule**: `surrogate-control` and `surrogate-kernel` depend only on `surrogate-contract` — never on each other. The bridge layer depends only on the contract. This ensures the data plane never blocks on the control plane.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+ (edition 2024)
- Linux or macOS

### Build

```bash
git clone https://github.com/YoRHa-Agents/Surrogate.git
cd Surrogate
cargo build --release
```

The binary will be at `target/release/surrogate-app`.

### Run

1. Create a configuration file (see [`examples/basic.toml`](examples/basic.toml)):

```toml
listen = "127.0.0.1:41080"
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"

[[outbounds]]
id = "reject"
type = "reject"

[[rules]]
id = "block-ads"
host_suffix = "ads.example.com"
outbound = "reject"

[[rules]]
id = "allow-local"
host_equals = "localhost"
port = 8080
outbound = "direct"
```

2. Validate your config:

```bash
surrogate-app validate-config examples/basic.toml
```

3. Start the proxy:

```bash
surrogate-app serve examples/basic.toml
```

4. Use the proxy:

```bash
# HTTP CONNECT
curl -x http://127.0.0.1:41080 https://example.com

# SOCKS5
curl --socks5 127.0.0.1:41080 https://example.com
```

### Test

```bash
cargo test
```

## Configuration

See the [Configuration Guide](docs/guide/configuration.md) for the full reference.

| Field | Type | Description |
|-------|------|-------------|
| `listen` | `string` | Socket address to listen on (e.g. `"127.0.0.1:41080"`) |
| `default_outbound` | `string` | Outbound ID used when no rule matches |
| `outbounds` | `array` | List of outbound transports (`direct` or `reject`) |
| `rules` | `array` | Ordered routing rules with matchers and outbound targets |

### Rule Matchers

Rules are evaluated in order. The first match wins.

| Matcher | Description |
|---------|-------------|
| `host_equals` | Exact hostname match (case-insensitive) |
| `host_suffix` | Hostname suffix match with dot boundary (e.g. `example.com` matches `sub.example.com`) |
| `port` | Exact destination port match |

Matchers within a rule are combined with AND logic.

## Project Status

Surrogate is in active development. The current v0.1.0 release provides:

| Component | Status |
|-----------|--------|
| HTTP CONNECT proxy | Working |
| SOCKS5 CONNECT proxy | Working |
| Rule-based routing | Working (host exact, suffix, port) |
| Observability events | Working |
| Connection pool | Implemented (not yet wired) |
| Streaming layer (H2/WS) | Scaffolded |
| Protocol handlers (SS/VLESS/Trojan/VMess/WG) | Scaffolded |
| Linux explicit proxy bridge | Partial (listener, no kernel handoff) |
| macOS Network Extension | Stub |
| Config import (Clash/sing-box/V2Ray) | Stub (IR-based) |
| Plugin system | Stub (6 built-in plugins registered) |

See the [User Guide](docs/guide/user-guide.md) for detailed documentation.

## Development

### Project Structure

```
Surrogate/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── surrogate-contract/ # Shared types and traits
│   ├── surrogate-kernel/   # Data plane proxy engine
│   ├── surrogate-control/  # Control plane services
│   ├── surrogate-bridge/   # Platform traffic capture
│   └── surrogate-app/      # CLI binary
├── docs/
│   ├── clean-room/         # Protocol implementation evidence
│   └── guide/              # User documentation
└── examples/               # Example configurations
```

### Running Specific Crate Tests

```bash
cargo test -p surrogate-contract   # 55 tests
cargo test -p surrogate-kernel     # 42 tests (38 unit + 4 integration)
cargo test -p surrogate-control    # 42 tests
cargo test -p surrogate-bridge     # 10 tests
```

### Clean-Room Protocol Policy

All protocol implementations follow a strict clean-room process:
- Source material limited to public documentation, packet captures, interop tests, and third-party writeups
- No reference implementation source code
- Evidence logs maintained in `docs/clean-room/`
- See [P-gate-0](docs/clean-room/README.md) for details

## License

MIT — see [Cargo.toml](Cargo.toml) for details.
