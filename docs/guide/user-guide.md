# Surrogate User Guide

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Commands](#cli-commands)
- [Proxy Protocols](#proxy-protocols)
- [Routing Rules](#routing-rules)
- [Observability](#observability)
- [Architecture Overview](#architecture-overview)
- [Troubleshooting](#troubleshooting)

---

## Installation

### From Source (recommended)

```bash
# Install Rust (1.85+ required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/YoRHa-Agents/Surrogate.git
cd Surrogate
cargo build --release

# Binary is at target/release/surrogate-app
```

### From GitHub Releases

Download pre-built binaries from the [Releases page](https://github.com/YoRHa-Agents/Surrogate/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x86_64 | `surrogate-app-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | aarch64 | `surrogate-app-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Apple Silicon | `surrogate-app-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `surrogate-app-x86_64-apple-darwin.tar.gz` |

```bash
# Example: Linux x86_64
tar xzf surrogate-app-x86_64-unknown-linux-gnu.tar.gz
chmod +x surrogate-app
./surrogate-app --help
```

---

## Quick Start

1. **Create a config file** (`proxy.toml`):

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
```

2. **Validate** the config:

```bash
surrogate-app validate-config proxy.toml
# Output: config valid: proxy.toml
```

3. **Start** the proxy:

```bash
surrogate-app serve proxy.toml
# Output: proxy kernel listening on 127.0.0.1:41080
```

4. **Connect** through the proxy:

```bash
# HTTP CONNECT tunnel
curl -x http://127.0.0.1:41080 https://httpbin.org/ip

# SOCKS5
curl --socks5 127.0.0.1:41080 https://httpbin.org/ip
```

5. **Stop** with `Ctrl+C`.

---

## CLI Commands

### `validate-config <path>`

Parse and validate a TOML configuration file without starting the proxy.

```bash
surrogate-app validate-config config.toml
```

Checks:
- Valid TOML syntax
- Valid socket address for `listen`
- At least one outbound defined
- Default outbound exists in outbound list
- No duplicate outbound or rule IDs
- All rule outbound references are valid
- All rules have at least one matcher

### `dump-normalized <path>`

Load a config, normalize it (lowercase IDs, sort outbounds, assign priorities), and print as JSON.

```bash
surrogate-app dump-normalized config.toml
```

Useful for debugging rule priority assignment and verifying normalization behavior.

### `serve <path>`

Start the proxy kernel. The proxy listens for both HTTP CONNECT and SOCKS5 connections on the configured address. Protocol detection is automatic based on the first byte of each connection.

```bash
surrogate-app serve config.toml
```

The proxy emits JSON-formatted observability events to stdout.

---

## Proxy Protocols

### HTTP CONNECT

Standard HTTP CONNECT tunneling (RFC 7231 §4.3.6). The proxy:

1. Receives a `CONNECT host:port HTTP/1.1` request
2. Evaluates routing rules against the target host and port
3. If the outbound is `direct`: connects to the upstream, responds with `200 Connection Established`, and relays data bidirectionally
4. If the outbound is `reject`: responds with `403 Forbidden`

### SOCKS5

SOCKS5 protocol (RFC 1928) with no-authentication method. The proxy:

1. Performs version/method negotiation (only `NO AUTHENTICATION` supported)
2. Reads the CONNECT command with the target address (IPv4, IPv6, or domain name)
3. Evaluates routing rules
4. Connects to the upstream or rejects based on the matched outbound

**Supported**: CONNECT command with IPv4, IPv6, and domain addresses.
**Not yet supported**: BIND, UDP ASSOCIATE.

### Protocol Detection

Surrogate automatically detects the protocol by peeking at the first byte:
- `0x05` → SOCKS5 handshake
- Anything else → HTTP CONNECT request

---

## Routing Rules

Rules are defined in the `[[rules]]` array and evaluated in config order. The first matching rule determines the outbound. If no rule matches, the `default_outbound` is used.

### Matchers

| Matcher | Type | Description |
|---------|------|-------------|
| `host_equals` | `string` | Exact match on target hostname (case-insensitive) |
| `host_suffix` | `string` | Suffix match with dot boundary — `example.com` matches `sub.example.com` but not `notexample.com` |
| `port` | `integer` | Exact match on destination port |

Multiple matchers in a single rule are combined with AND logic. At least one matcher is required per rule.

### Outbound Types

| Type | Behavior |
|------|----------|
| `direct` | Forward traffic directly to the target |
| `reject` | Drop the connection (HTTP 403 / SOCKS5 general failure) |

### Example

```toml
# Block all subdomains of ads.example.com
[[rules]]
id = "block-ads"
host_suffix = "ads.example.com"
outbound = "reject"

# Allow specific host on specific port
[[rules]]
id = "allow-api"
host_equals = "api.myservice.com"
port = 443
outbound = "direct"
```

---

## Observability

The proxy emits JSON events to stdout for every connection lifecycle stage:

```json
{"kind":"session_started","session_id":1,"message":"session started","host":"example.com","port":443}
{"kind":"rule_matched","session_id":1,"message":"rule matched","host":"example.com","port":443,"rule_id":"default","outbound":"direct"}
{"kind":"forward_connected","session_id":1,"message":"forward connected","host":"example.com","port":443,"outbound":"direct"}
{"kind":"forward_closed","session_id":1,"message":"forward closed","bytes_from_client":142,"bytes_from_upstream":5678}
```

### Event Types

| Event | When |
|-------|------|
| `session_started` | New connection accepted, target parsed |
| `rule_matched` | Routing rule evaluated |
| `forward_connected` | Upstream connection established |
| `forward_closed` | Relay complete, includes byte counters |
| `error` | Any error during processing |

---

## Architecture Overview

Surrogate uses a five-crate workspace architecture:

### surrogate-contract (shared types)

All domain types, traits, and error definitions live here. This is the only crate that `surrogate-kernel` and `surrogate-control` depend on — they never depend on each other.

Key types: `NormalizedConfig`, `CompiledRuleSet`, `Event`, `Observability`, `PluginHandle`, `HealthProbe`, `Transport`.

### surrogate-kernel (data plane)

The performance-critical proxy engine. Handles TCP accept loop, protocol detection, HTTP CONNECT / SOCKS5 parsing, rule matching, and bidirectional data relay.

### surrogate-control (control plane)

Configuration management, rule compilation, plugin lifecycle, config import from external formats, test workbench, and coverage analysis.

### surrogate-bridge (platform layer)

Platform-specific traffic capture. On Linux, provides an explicit proxy listener. macOS Network Extension support is planned.

### surrogate-app (CLI)

Thin CLI binary that wires the crates together and provides the `validate-config`, `dump-normalized`, and `serve` commands.

---

## Troubleshooting

### "listen address is not a valid socket address"

Ensure your `listen` field is a valid `host:port` pair:

```toml
listen = "127.0.0.1:41080"   # Correct
listen = "localhost:41080"    # Wrong — must be an IP address
```

### "default outbound does not exist"

The `default_outbound` must match an outbound `id` (case-insensitive):

```toml
default_outbound = "direct"

[[outbounds]]
id = "direct"
type = "direct"
```

### "rule must contain at least one matcher"

Every rule needs at least one of `host_equals`, `host_suffix`, or `port`.

### Connection refused

Check that the proxy is running and listening on the expected address. The `serve` command prints the bound address on startup.
