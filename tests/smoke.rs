// tests/smoke.rs — mcp-probe CLI smoke tests

use std::process::Command;

fn bin() -> String {
    let target = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    format!("{target}/release/mcp-probe")
}

#[test]
fn help_lists_all_commands() {
    let out = Command::new(bin()).arg("--help").output().expect("spawn");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let s = String::from_utf8_lossy(&out.stdout);
    for cmd in ["Ping", "List", "Call", "Inspect"] {
        assert!(s.contains(cmd), "missing command {cmd} in help:\n{s}");
    }
}

#[test]
fn ping_without_server_fails_gracefully() {
    // No MCP_SERVER_CMD set — should error, not panic
    let out = Command::new(bin()).arg("ping").env_clear().output().expect("spawn");
    assert!(!out.status.success(), "expected failure, got success");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("MCP_SERVER_CMD") || err.contains("error"), "unexpected stderr: {err}");
}
