# mcp-probe

A **curl for MCP servers**. Ping, list tools, call them, inspect the wire format — without writing a full MCP client.

MCP (Model Context Protocol) servers speak JSON-RPC 2.0 over stdio or HTTP. `mcp-probe` gives you a single binary to debug them interactively.

## Install

```bash
cargo install mcp-probe
```

## Quick start

Set `MCP_SERVER_CMD` to the command that starts your MCP server, then:

```bash
export MCP_SERVER_CMD="npx -y @modelcontextprotocol/server-filesystem /tmp"

mcp-probe ping
mcp-probe list
mcp-probe call read_file '{"path": "/tmp/test.txt"}'
mcp-probe inspect --method tools/list
```

## Commands

| Command | What it does |
|---|---|
| `ping` | Send a ping, measure latency |
| `list` | Enumerate all tools the server exposes |
| `call <name>` | Call a tool by name with JSON arguments |
| `inspect` | Dump the raw JSON-RPC request/response pair |

## Flags

- `--server <name>` — server name (currently only `stdio` is supported in v0.1)
- `--timeout <secs>` — ping timeout (default 10s)
- `--args <json>` — JSON arguments for `call`
- `--method <name>` — JSON-RPC method for `inspect`

## How it works

`mcp-probe` spawns your MCP server as a child process, communicates over stdin/stdout using newline-delimited JSON-RPC 2.0, and pretty-prints the results. No daemon, no state — just one-shot requests.

## License

MIT. See [LICENSE](LICENSE).
