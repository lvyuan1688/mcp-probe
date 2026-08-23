// mcp-probe: MCP server 的调试工具
// 支持 stdio 和 HTTP transport,能 ping/list tools/call tool/inspect wire format

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::ExitCode;
use std::time::Duration;

mod transport;
mod wire;

/// A curl for MCP servers
#[derive(Parser, Debug)]
#[command(name = "mcp-probe", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send a ping request to verify the server is alive
    Ping {
        /// Server name (from config) or stdio command
        #[arg(long)]
        server: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
    /// List all tools the server exposes
    List {
        #[arg(long)]
        server: Option<String>,
    },
    /// Call a tool by name with JSON arguments
    Call {
        #[arg(long)]
        server: Option<String>,
        /// Tool name
        name: String,
        /// JSON arguments (string)
        #[arg(long)]
        args: Option<String>,
    },
    /// Show the raw wire format of the last request/response
    Inspect {
        #[arg(long)]
        server: Option<String>,
        /// JSON-RPC method to inspect
        #[arg(long, default_value = "tools/list")]
        method: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: failed to create tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = rt.block_on(async move {
        match &cli.command {
            Commands::Ping { server, timeout } => {
                let mut t = transport::connect(server.as_deref()).await?;
                let req = JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: 1,
                    method: "ping".into(),
                    params: None,
                };
                let resp = tokio::time::timeout(
                    Duration::from_secs(*timeout),
                    t.request(&req),
                )
                .await
                .map_err(|_| anyhow::anyhow!("ping timed out after {}s", timeout))??;
                println!("pong ({} ms)", resp.result.get("latency_ms").map(|v| v.as_u64().unwrap_or(0)).unwrap_or(0));
                Ok(())
            }
            Commands::List { server } => {
                let mut t = transport::connect(server.as_deref()).await?;
                let req = JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: 2,
                    method: "tools/list".into(),
                    params: None,
                };
                let resp = t.request(&req).await?;
                if let Some(tools) = resp.result.get("tools").and_then(|v| v.as_array()) {
                    println!("Found {} tools:", tools.len());
                    for tool in tools {
                        let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let desc = tool.get("description").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {name:<30} {desc}");
                    }
                } else {
                    println!("No tools field in response");
                }
                Ok(())
            }
            Commands::Call { server, name, args } => {
                let mut t = transport::connect(server.as_deref()).await?;
                let params = match args {
                    Some(s) => Some(serde_json::from_str(s).map_err(|e| anyhow::anyhow!("invalid args JSON: {e}"))?),
                    None => None,
                };
                let req = JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: 3,
                    method: "tools/call".into(),
                    params: Some(serde_json::json!({ "name": name, "arguments": params })),
                };
                let resp = t.request(&req).await?;
                println!("{}", serde_json::to_string_pretty(&resp.result).unwrap_or_default());
                Ok(())
            }
            Commands::Inspect { server, method } => {
                let mut t = transport::connect(server.as_deref()).await?;
                let req = JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: 4,
                    method: method.clone(),
                    params: None,
                };
                let raw_req = serde_json::to_string_pretty(&req).unwrap_or_default();
                let resp = t.request(&req).await?;
                let raw_resp = serde_json::to_string_pretty(&resp).unwrap_or_default();
                println!("=== Request ===\n{raw_req}\n\n=== Response ===\n{raw_resp}");
                Ok(())
            }
        }
    });

    if let Err(e) = result {
        eprintln!("error: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
