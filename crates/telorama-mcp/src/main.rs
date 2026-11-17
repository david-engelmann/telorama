//! The telorama MCP server binary: JSON-RPC 2.0 over stdio, one
//! message per line, delegating every message to the library.

use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = telorama_mcp::handle_message(&line) {
            writeln!(stdout, "{response}").expect("stdout must accept responses");
            stdout.flush().expect("stdout must flush");
        }
    }
}
