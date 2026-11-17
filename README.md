# telorama

telorama is a visualization engine for OpenTelemetry data. An AI agent describes the diagram it wants in a small structured document called a scene spec. telorama checks the spec against a set of honesty rules, works out the geometry itself, and renders a finished diagram that can appear inside an AI chat, a pull request, or a plain file.

## What it does

- It turns a trace into a waterfall diagram that shows where one request spent its time and which part of the work sits on the critical path.
- It summarizes a service with rate, error, and duration figures drawn from the same trace data, with duration reported as percentiles and never as an average.
- It draws service maps whose edges come only from calls the traces actually contain, with a ranked rate, error, and duration summary beneath the map.
- It draws latency heatmaps from declared histogram buckets and names one real span from the data as the exemplar of the busiest cell, never an invented link.
- It draws flame graphs from sampled profiles, and it checks that a profile carries exactly the sample count the spec declares before it renders.
- It turns an agent session into a run diagram that shows each turn, each model call and tool call, the time spent waiting on permissions, and what the session cost in tokens and money.
- It renders every diagram deterministically. The same spec and the same data always produce the same file, byte for byte, so a rendered diagram can be checked in tests and compared across changes.
- It prints the source, the query, the time window, and the sampling state on every scene, so a reader can tell where each number came from.

## How it is used

The command line tool reads a scene spec plus the data its panels name and writes an SVG image. The data can be an OpenTelemetry trace export in OTLP JSON form, a folded stack profile, or an agent session export of OTLP logs and spans.

```text
telorama render --scene scene.json --traces traces.json --out scene.svg
telorama render --scene flame.json --profile stacks.txt --out flame.svg
telorama render --scene run.json --session-logs logs.json --session-spans spans.json --out run.svg
telorama validate scene.json
telorama inspect traces.json
telorama session --logs logs.json --spans spans.json
```

The validate command checks a scene spec against the schema and the honesty rules without rendering it. The inspect command summarizes a trace file by service, span count, and time range, so an agent can learn what data exists before it writes a spec. The session command prints an agent run as JSON for tools that read text rather than pictures. The Claude Code dialect is the first session dialect supported.

Later milestones add an MCP server that returns each scene as text for the model, structured data, and an interactive view for chat surfaces that can render one. The scene spec stays the same across every surface.

## Current state

Milestones 1 and 2 are complete. The scene spec, the honesty validator, the field system of units and thresholds, the OTLP trace and profile readers, the Claude Code session adapter, and the deterministic SVG renderer are in place. The renderer covers waterfall, stat, and RED panels, service maps, latency heatmaps, flame graphs, and agent run diagrams, with a golden test corpus built from realistic fixtures in the shape of the OpenTelemetry Demo and a recorded Claude Code session. Sankey panels are specified and validated but have no renderer yet. The MCP server is the next milestone and does not exist yet.

## Licence

telorama is available under the MIT licence or the Apache License, Version 2.0, at your choice.
