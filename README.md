# telorama

telorama is a visualization engine for OpenTelemetry data. An AI agent describes the diagram it wants in a small structured document called a scene spec. telorama checks the spec against a set of honesty rules, works out the geometry itself, and renders a finished diagram that can appear inside an AI chat, a pull request, or a plain file.

## What it does

- It turns a trace into a waterfall diagram that shows where one request spent its time and which part of the work sits on the critical path.
- It summarizes a service with rate, error, and duration figures drawn from the same trace data, with duration reported as percentiles and never as an average.
- It renders every diagram deterministically. The same spec and the same data always produce the same file, byte for byte, so a rendered diagram can be checked in tests and compared across changes.
- It prints the source, the query, the time window, and the sampling state on every scene, so a reader can tell where each number came from.

## How it is used

The command line tool reads a scene spec and an OpenTelemetry trace export in OTLP JSON form and writes an SVG image.

```text
telorama render --scene scene.json --traces traces.json --out scene.svg
telorama validate scene.json
telorama inspect traces.json
```

The validate command checks a scene spec against the schema and the honesty rules without rendering it. The inspect command summarizes a trace file by service, span count, and time range, so an agent can learn what data exists before it writes a spec.

Later milestones add an MCP server that returns each scene as text for the model, structured data, and an interactive view for chat surfaces that can render one. The scene spec stays the same across every surface.

## Current state

Milestone 1 is open. The scene spec, the honesty validator, the OTLP trace file reader, and the deterministic SVG renderer for waterfall, stat, and RED panels are in place, with a golden test corpus built from realistic trace fixtures in the shape of the OpenTelemetry Demo. Service maps, flame graphs, heatmaps, the field system, and the MCP server are later milestones and do not exist yet.

## Licence

telorama is available under the MIT licence or the Apache License, Version 2.0, at your choice.
