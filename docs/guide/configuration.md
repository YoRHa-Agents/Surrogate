# Configuration Reference

Surrogate uses a TOML configuration file. This document describes every field.

## Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `listen` | `string` | Yes | Socket address to bind the proxy listener. Must be a valid `IP:port` pair. |
| `default_outbound` | `string` | Yes | ID of the outbound used when no routing rule matches. Must reference an existing outbound. |
| `outbounds` | `array of table` | Yes (≥1) | Outbound transport definitions. |
| `rules` | `array of table` | No | Ordered routing rules. |

## `[[outbounds]]`

Each outbound defines a transport strategy for forwarded traffic.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Unique identifier (case-insensitive after normalization). |
| `type` | `string` | Yes | Transport type: `"direct"` or `"reject"`. |

### Outbound Types

**`direct`** — Open a TCP connection to the target and relay data bidirectionally.

**`reject`** — Immediately refuse the connection. HTTP CONNECT receives `403 Forbidden`; SOCKS5 receives `REP_GENERAL_FAILURE`.

## `[[rules]]`

Rules are evaluated in the order they appear in the config file. The first matching rule determines the outbound for a connection. If no rule matches, `default_outbound` is used.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Unique rule identifier (case-insensitive). |
| `host_equals` | `string` | No | Exact hostname match (case-insensitive). |
| `host_suffix` | `string` | No | Hostname suffix match with dot boundary. `example.com` matches `sub.example.com` but not `notexample.com`. |
| `port` | `integer` | No | Exact destination port match (1–65535). |
| `outbound` | `string` | Yes | Target outbound ID. Must reference an existing outbound. |

At least one matcher (`host_equals`, `host_suffix`, or `port`) is required per rule.

When multiple matchers are present in a single rule, they are combined with **AND** logic — all must match.

## Normalization

When the config is loaded, the following normalization is applied:

1. All IDs and hostnames are lowercased
2. Outbounds are sorted alphabetically by ID
3. Rules are assigned sequential priority values (1, 2, 3, ...) based on config order
4. Host suffix leading dots are stripped (`.example.com` → `example.com`)

Use `surrogate-app dump-normalized <config>` to inspect the normalized output.

## Validation Rules

The config loader performs these checks:

- `listen` must parse as a valid `SocketAddr`
- `default_outbound` must not be empty and must reference an existing outbound
- At least one outbound must be defined
- No duplicate outbound IDs
- No duplicate rule IDs
- No empty rule IDs
- Each rule must have at least one matcher
- Each rule's `outbound` must reference an existing outbound

## Example

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
id = "block-tracking"
host_suffix = "tracker.example.net"
outbound = "reject"

[[rules]]
id = "allow-local"
host_equals = "localhost"
outbound = "direct"

[[rules]]
id = "dev-server"
port = 8080
outbound = "direct"
```
