# Telorama plugin

This directory packages telorama for Claude Code as a plugin. It
registers the telorama MCP server and ships the telorama skill,
which teaches an agent when to render what.

![An agent state panel drawn from a working agent's own report](<../docs/media/agent-state.png>)

## What you need first

The plugin starts a binary named `telorama-mcp`, so that binary
has to be on your PATH before the plugin can serve. Build it from
the repository root with cargo.

```sh
cargo build --release -p telorama-mcp
```

Put `target/release/telorama-mcp` somewhere on your PATH, or
install it into your cargo bin directory.

```sh
cargo install --path crates/telorama-mcp
```

## Installing the plugin

Point Claude Code at this directory with its plugin flag, or copy
the directory into your Claude Code plugins directory. The
manifest lives in `.claude-plugin/plugin.json`, the server
registration in `.mcp.json`, and the skill in
`skills/telorama/SKILL.md`.

## What it gives the agent

Seven MCP tools covering scene validation, trace inspection, scene
rendering with the triple contract, agent session adaptation
across the Claude Code, Codex, GenAI, and OpenInference dialects,
incident composition, the session frame stream, and an app-only
panel drill-down for hosts that render MCP Apps.
