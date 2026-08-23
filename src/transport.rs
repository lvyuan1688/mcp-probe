// transport.rs: stdio / HTTP MCP transport

use crate::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command as TokioCommand};

pub enum Transport {
    Stdio(StdioTransport),
}

pub async fn connect(server: Option<&str>) -> Result<Transport> {
    let s = server.unwrap_or("stdio");
    match s {
        "stdio" => {
            // 默认 stdio,通过子进程启动 MCP server
            // 这里假设 server 命令由环境变量 MCP_SERVER_CMD 提供
            let cmd = std::env::var("MCP_SERVER_CMD")
                .map_err(|_| anyhow::anyhow!("set MCP_SERVER_CMD to the command that starts your MCP server, e.g. \"npx -y @modelcontextprotocol/server-filesystem /tmp\""))?;
            Ok(Transport::Stdio(StdioTransport::spawn(&cmd)?))
        }
        other => Err(anyhow::anyhow!("unsupported transport: {other} (only 'stdio' supported in v0.1)")),
    }
}

pub struct StdioTransport {
    child: Child,
    stdin: ChildStdin,
    stdout_rx: tokio::sync::mpsc::Receiver<JsonRpcResponse>,
    next_id: AtomicU64,
}

impl StdioTransport {
    fn spawn(cmd: &str) -> Result<Self> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let mut c = TokioCommand::new(parts[0]);
        c.args(&parts[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let mut child = c.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;

        // 启动一个 task 读 stdout,把每行 JSON 推到 channel
        let (tx, rx) = tokio::sync::mpsc::channel::<JsonRpcResponse>(16);
        let mut reader = AsyncBufReader::new(stdout);
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,  // EOF
                    Ok(_) => {
                        if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&line) {
                            if tx.send(resp).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stdout_rx: rx,
            next_id: AtomicU64::new(1),
        })
    }

    pub async fn request(&mut self, req: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let raw = serde_json::to_string(req)? + "\n";
        self.stdin.write_all(raw.as_bytes()).await?;
        self.stdin.flush().await?;

        // 等对应 id 的 response
        let target_id = req.id;
        while let Some(resp) = self.stdout_rx.recv().await {
            if resp.id == target_id {
                return Ok(resp);
            }
            // 其他 id 的 response,跳过(简化处理)
        }
        Err(anyhow::anyhow!("no response for id {}", target_id))
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // kill_on_drop 会处理
    }
}

// 给 main.rs 用的简化 stub,避免 unused import 警告
#[allow(dead_code)]
fn _unused(_: Value) {}
