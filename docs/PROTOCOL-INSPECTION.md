# Protocol Inspection

mcp-probe inspects MCP (Model Context Protocol) traffic between client and server for
debugging and development.

## Capture modes

| Mode | How | Use case |
|---|---|---|
| Proxy | Sits between client and server, transparent | Production debugging |
| Attach | Hooks into existing Unix socket / pipe | Local dev |
| Replay | Replays recorded messages against a server | Regression testing |

## Message format

Captured messages are stored as JSONL:

```json
{"ts":"2026-09-03T10:00:00Z","dir":"c2s","method":"tools/call","id":42,"params":{"name":"read_file","args":{"path":"x.rs"}}}
{"ts":"2026-09-03T10:00:01Z","dir":"s2c","id":42,"result":{"content":"fn main(){}"}}
```

## Analysis

- **Latency** -- per-request round-trip time, p50/p95/p99.
- **Errors** -- JSON-RPC error codes, frequency, stack traces.
- **Schema validation** -- each message validated against MCP spec.
- **Traffic summary** -- method frequency, token counts, active tools.
