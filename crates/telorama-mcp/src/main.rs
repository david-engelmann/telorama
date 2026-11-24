//! The telorama MCP server binary: JSON-RPC 2.0 over stdio, one
//! message per line, delegating every message to the library.

use std::io::{BufRead, Write};

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut buf: Vec<u8> = Vec::new();
    loop {
        buf.clear();
        match stdin.read_until(b'\n', &mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        // A line that is not valid UTF-8 cannot be a JSON-RPC
        // message. It earns a parse error and the server keeps
        // serving, rather than exiting in silence (INV-88).
        let line = match std::str::from_utf8(&buf) {
            Ok(s) => s,
            Err(_) => {
                let response = "{\"jsonrpc\":\"2.0\",\"id\":null,\"error\":{\"code\":-32700,\"message\":\"Parse error: the line is not valid UTF-8\"}}";
                writeln!(stdout, "{response}").expect("stdout must accept responses");
                stdout.flush().expect("stdout must flush");
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(response) = telorama_mcp::handle_message(line) {
            writeln!(stdout, "{response}").expect("stdout must accept responses");
            stdout.flush().expect("stdout must flush");
        }
    }
}
