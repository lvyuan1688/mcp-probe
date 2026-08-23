// wire.rs: MCP wire format helpers
// MCP 用 JSON-RPC 2.0 over stdio/HTTP,每行一个 JSON 对象

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 标准 JSON-RPC 2.0 请求
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// 标准 JSON-RPC 2.0 响应
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP 协议初始化握手
pub fn initialize_request(id: u64) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "initialize".into(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "mcp-probe", "version": "0.1.0" }
        })),
    }
}

/// 列出所有工具
pub fn list_tools_request(id: u64) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "tools/list".into(),
        params: None,
    }
}

/// 调用工具
pub fn call_tool_request(id: u64, name: &str, args: Option<Value>) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "tools/call".into(),
        params: Some(serde_json::json!({
            "name": name,
            "arguments": args.unwrap_or(Value::Null)
        })),
    }
}

/// 标准错误码
pub mod error_codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
}
