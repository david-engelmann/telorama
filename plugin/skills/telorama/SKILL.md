---
name: telorama
description: Render OpenTelemetry traces, service health, latency, profiles, and agent sessions as deterministic diagrams with the telorama MCP server. Use when a question about telemetry data is better answered by a picture than by prose, when an incident needs a first scene, or when an agent session needs its run shown rather than described.
---

# Telorama

Telorama turns telemetry into scenes. A scene is a small validated
spec that names its data, its panels, and the question it answers.
The renderer is deterministic, so the same spec over the same data
produces the same SVG byte for byte, and every number printed on a
scene traces back to the data it names.

## The tools

- `validate_scene` checks a scene spec against the honesty rules
  before anything renders. Run it first whenever you author a scene
  by hand.
- `inspect_traces` summarizes an OTLP trace export, counts, window,
  and per-service errors, before you commit to a scene.
- `render_scene` renders a scene and returns the triple contract,
  a concise text answer for the model, compact structured content,
  and the SVG for the view on hosts that render MCP Apps.
- `session_run` adapts an agent session export into the canonical
  run model and returns it as JSON.
- `incident_scene` composes the incident template from a trace
  export and returns the triple contract plus the composed spec.
- `scene_panel_detail` is app-only and serves the view's
  drill-down. The model never calls it directly.

## When to render what

One trace under investigation wants a waterfall. Author a scene
with a waterfall panel over the trace, critical path and errors
highlighted, and add a stat panel when a single number answers the
question.

Service health across a window wants the RED row and the service
map together. The row carries rate, errors, and duration per
service, and the map shows who calls whom. Map edges exist only
where a span's parent belongs to another service in the same
trace, so the picture never invents a dependency.

Latency shape wants a heatmap. Declare the bucket edges on the
data entry, because buckets are data, not style. The rendered grid
names a real span as its exemplar, the longest span in the busiest
cell.

Profile data wants a flame panel. Declare the sample count and let
the validator check it against the folded stacks. A flame built
from span self time is a labelled fallback and says so on its
face.

An agent session wants the run panel. Claude Code and Codex
sessions arrive as OTLP logs, the first with spans to fuse for
permission waits and subagents. GenAI and OpenInference sessions
arrive as spans alone. Call `session_run` with the matching
dialect to read the canonical run, then place an agent_run panel
in a scene when the run belongs in a picture.

An incident with a known window wants `incident_scene` before any
hand authoring. The template composes a service map, the RED row,
a waterfall of the exemplar trace, and a latency heatmap, and it
derives the headline and narrative from the same numbers the
panels print. Its choice rules are fixed and stated in the
template, worst service by error ratio, exemplar the longest
erroring trace, so the first scene of an incident is one an
investigator can check rather than trust.

## The honesty rules

Percentiles are percentiles and never averages. A stat names the
percentile it reports.

Every panel carries provenance. The rendered caption states the
window, the data entries, and who generated the scene and why.

Sampling is always stated. A scene that does not know says
unknown.

Cost is printed only when the dialect exports it or the render
prices it in the open. Claude Code exports cost. Codex, GenAI, and
OpenInference export tokens without cost, so telorama prices those
tokens from its own table and the render says the figure is an
estimate, names the table date, and names any model it could not
price rather than folding it in as zero.

Color is never the only channel. Errors carry a text marker as
well as the vermillion.

## Hosts

Hosts that render MCP Apps show the view, and the app-only
drill-down keeps bulk detail out of the model's context. Terminal
hosts such as Claude Code and Codex render nothing, and there the
text and structured content are the product, written to be read.
The CLI's render-html command produces the same scene as a
standalone page when a person needs the picture outside a chat.
