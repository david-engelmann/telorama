# telorama

telorama is a visualization engine for OpenTelemetry data. An AI agent describes the diagram it wants in a small structured document called a scene spec. telorama checks the spec against a set of honesty rules, works out the geometry itself, and renders a finished diagram that can appear inside an AI chat, a pull request, or a plain file.

## What it does

- It turns a trace into a waterfall diagram that shows where one request spent its time and which part of the work sits on the critical path.
- It summarizes a service with rate, error, and duration figures drawn from the same trace data, with duration reported as percentiles and never as an average.
- It draws service maps whose edges come only from calls the traces actually contain, with a ranked rate, error, and duration summary beneath the map.
- It draws latency heatmaps from declared histogram buckets and names one real span from the data as the exemplar of the busiest cell, never an invented link.
- It draws flame graphs from sampled profiles, and it checks that a profile carries exactly the sample count the spec declares before it renders.
- It turns an agent session into a run diagram that shows each turn, each model call and tool call, the time spent waiting on permissions, and what the session cost in tokens and money.
- It reads Codex session exports as well as Claude Code ones. Codex reports no cost of its own, so telorama prices the tokens itself and prints on the diagram that the figure is its own estimate.
- It renders every diagram deterministically. The same spec and the same data always produce the same file, byte for byte, so a rendered diagram can be checked in tests and compared across changes.
- It prints the source, the query, the time window, and the sampling state on every scene, so a reader can tell where each number came from.
- It serves scenes over MCP as short text a model can read, structured figures, and an interactive view for chat surfaces that can render one, and it can write any scene as a single standalone HTML file with nothing left to fetch.

## How it is used

The command line tool reads a scene spec plus the data its panels name and writes an SVG image. The data can be an OpenTelemetry trace export in OTLP JSON form, a folded stack profile, or an agent session export of OTLP logs and spans.

```text
telorama render --scene scene.json --traces traces.json --out scene.svg
telorama render --scene flame.json --profile stacks.txt --out flame.svg
telorama render --scene run.json --session-logs logs.json --session-spans spans.json --out run.svg
telorama render-html --scene scene.json --traces traces.json --out scene.html
telorama validate scene.json
telorama inspect traces.json
telorama session --logs logs.json --spans spans.json
telorama session --logs codex-logs.json --dialect codex
```

The validate command checks a scene spec against the schema and the honesty rules without rendering it. The inspect command summarizes a trace file by service, span count, and time range, so an agent can learn what data exists before it writes a spec. The session command prints an agent run as JSON for tools that read text rather than pictures. Two session dialects are supported, Claude Code and Codex.

The same engine also runs as an MCP server, the telorama-mcp program, speaking JSON-RPC over standard input and output. A host connects it once and its tools validate a spec, summarize a trace export, adapt a session, and render a scene. A render returns short text the model can read, structured figures drawn from the same workings as the picture, and the picture itself for hosts that can show the interactive view. One further tool exists only for that view and fetches the detail behind a single panel, so drilling down costs the model no context. The scene spec stays the same across every surface.

## Current state

Milestones 1, 2, and 3 are complete. The scene spec, the honesty validator, the field system of units and thresholds, the OTLP trace and profile readers, the Claude Code and Codex session adapters, the deterministic SVG renderer, the standalone HTML export, and the MCP server with its interactive view are in place. The renderer covers waterfall, stat, and RED panels, service maps, latency heatmaps, flame graphs, and agent run diagrams, with a golden test corpus built from realistic fixtures in the shape of the OpenTelemetry Demo and recorded agent sessions in both dialects. Sankey panels are specified and validated but have no renderer yet. Milestone 4, incident composition and packaging for publication, is next.

## Licence

telorama is available under the MIT licence or the Apache License, Version 2.0, at your choice.
