# New flagship: OTel-native embed-first visualization engine — concept dossier

Synthesized 2026-11-16 from a six-front deep dive. The six research fronts behind it, covering OTel diagrams, agent telemetry, embed surfaces, prior art, the Grafana bar, and rendering technology, are kept outside this repository. Claims marked UNVERIFIED in the fronts stay
unverified here.

## Thesis

Every observability vendor's MCP server stops at data and deeplinks to its own
UI. Every chart MCP server wraps a JS library and returns a static PNG of a
generic business chart. Nobody ships a general engine that composes bespoke,
Grafana-class telemetry visualizations per question and renders them inside
the AI chat itself. This repo is that engine: OpenTelemetry-native semantics,
an agent-authored scene spec, a deterministic Rust renderer, every host tier.

The flagship diagram family is the agent-run waterfall. Claude Code exports a
nested OTLP span tree (interaction, LLM request, tool with permission-wait
sub-spans, subagents nested under the spawning tool span); Codex emits all
three signals in its own dialect; the GenAI semantic conventions define the
shared vocabulary but are Development-status throughout with no release to
pin. A dialect-normalizing ingest that fuses traces, logs and metrics into one
canonical agent-run model is the moat, and the engine's own MCP server can
render the run the user is currently inside. The demo is the product.

## Established facts the design rests on

- MCP Apps (SEP-1865, io.modelcontextprotocol/ui) is Stable as of 2026-01-26.
  A tool declares _meta.ui.resourceUri pointing at a ui:// HTML resource; the
  host renders it in a sandboxed iframe; data flows over a JSON-RPC bridge.
- Rendering hosts as of 2026-11-25 are Claude web, desktop and mobile,
  ChatGPT, VS Code Copilot, Microsoft 365 Copilot, Cursor, and Goose on
  the desktop. Claude Code CLI and Codex CLI do NOT render apps. They
  consume text and structuredContent. Terminal output is a first-class
  tier, not a fallback afterthought.
- Default CSP for a view is deny-by-default with connect-src 'none'. The
  working shape is one self-contained HTML file, zero declared domains, all
  data arriving via the tool result or host-proxied app-only tool calls.
- Grafana's MCP server renders telemetry through MCP Apps only as one generic
  query chart at a time. Elastic ships MCP Apps as fixed product surfaces
  bound to Elastic data. autotel-devtools renders one hard-coded local trace
  waterfall in chat. Microsoft Flint (2026-10-19) proves the small semantic
  spec plus deterministic compiler architecture for generic charts.
- Grafana's quality is a system: a field-configuration layer (units,
  decimals, thresholds, by-name colors) applied identically everywhere;
  thresholds as verdicts; legend tables carrying calculations; shared
  crosshair across panels; the CRISP critical-path overlay on waterfalls;
  service-graph edges only from CLIENT/SERVER and PRODUCER/CONSUMER span
  pairs; exemplars joining a metric spike to a real trace.
- Answer-shaped beats dashboard-shaped in chat: headline takeaway first with
  the key number in it, one question per scene, one to three panels, query
  and window and sampling printed as provenance caption, annotations on the
  data, interaction local and shallow over baked data.

## Architecture

Pipeline, all Rust, one direction:

SceneSpec (JSON, agent-authored, versioned) → validate and canonicalize →
derive (pure functions) → layout → DisplayList (all text pre-measured and
positioned, content-hash element IDs, canonical float formatting) → emitters.

Crates:
- scene-core: spec types, JSON Schema, validator including the honesty rules,
  and the derivations: span tree, critical path (CRISP), self-time, edge
  aggregation with RED per edge, histogram bucketing from real le buckets,
  conserved flow builder, flame folding (profiles, plus a labelled span
  self-time fallback), exemplar join. Correctness lives here.
- ingest: OTLP traces, logs and metrics; dialect adapters for claude_code,
  codex, the gen_ai generations (1.30, 1.37, 1.41 shapes), OpenInference and
  Vercel ai.*; signal fusion into two canonical models (service telemetry,
  agent run). Backend adapters: Prometheus, Tempo, Loki, Pyroscope, Jaeger
  over HTTP, plus bring-your-own frames and OTLP files. Not a datastore, not
  a query language.
- layout: hand-rolled span Gantt, flame partition, heatmap bucketing;
  Sugiyama layered layout behind a trait, in-house at service-map scale
  (rust-sugiyama carries an UNVERIFIED hang report; graphviz-pure maturity
  UNVERIFIED; neither is a load-bearing default).
- text: fontdb plus harfrust over a build-time-subsetted embedded font; the
  same font bytes are measured server-side and shipped in the HTML shell.
  Default output is real text elements; a text-to-paths mode exists for
  archival SVG.
- svg: canonical writer over the DisplayList. Same spec plus same data plus
  same engine version gives byte-identical SVG and scene JSON, enforced by
  golden-file tests in CI.
- html: the MCP Apps shell. A pinned static shell (CSS consuming host theme
  variables with engine fallbacks, the subset font, a small hand-written
  scene-player JS that paints the DisplayList and wires pan, zoom, tooltips
  and the bridge) plus the scene JSON. Declares no CSP domains.
- mcp: the server. Model-facing tools stay few and opinionated:
  describe_data, query (returns frame summaries, never a firehose), render,
  render_panel, and parameterised scene templates (incident_scene,
  trace_scene, agent_run_scene, red_row, deploy_compare). App-only tools
  (visibility app) serve drill-down. Every tool returns the triple contract:
  concise text content, compact model-legible structuredContent, and the
  view. Fallback ladder from one spec: interactive app, self-contained HTML,
  SVG, PNG via resvg, Markdown table plus narrative.
- Palette: Okabe-Ito categorical, contrast-filtered at render time against
  the actual theme background (Grafana's own colorblind-mode trick);
  viridis or cividis for heatmaps; color never the only channel.

Honesty rules enforced by the validator, not by prompt discipline: durations
are percentiles, never averages; sankey flow is conserved with an explicit
residual bucket; heatmaps render real histogram buckets, no interpolated
surface; flame graphs declare sample counts and refuse below a floor;
exemplars that were never captured degrade to a window search, never a fake
link; every panel carries source, exact query, window and sampling state;
derived numbers show their derivation.

As built, the pipeline ships as five crates. telorama-scene carries the spec,
the schema, the honesty validator, and the derivations. telorama-ingest
carries the readers and the four dialect adapters. telorama-svg carries
layout, the character-class text widths, and the canonical SVG writer.
telorama-mcp carries the server, its seven tools, and the MCP Apps shell. The
telorama crate is the command line tool. No font library is embedded, because
layout measures text by character class under INV-92 rather than by font
metrics.

## Scope fences

No datastore. No query language. No dashboard editor, no alerting. No
hosted renderer; rendering is local and in-process with no network in the
render path. Generic business charts appear only as supporting panels inside
telemetry scenes. The scene spec stays small and semantic; agents never
supply geometry, decimals or CSS (the Flint and Grafana-JSON lesson).

## Milestones

- M1: scene spec v0 with schema and honesty validator; OTLP file ingest for
  traces; waterfall plus stat and RED panels to deterministic SVG; golden
  corpus from realistic fixtures (OTel Demo traces).
- M2: the field system (units, thresholds, legend tables, shared cursor);
  service map with RED table; flame from profiles with labelled span
  fallback; latency heatmap with exemplar overlay; agent-run model with the
  Claude Code dialect adapter, logs-first fusion.
- M3: MCP server with the triple output contract; MCP Apps view (shell plus
  scene player) verified rendered in Claude; drill-down through app-only
  tools; Codex dialect adapter with renderer-side token pricing; standalone
  HTML export for terminal hosts.
- M4: incident scene composition across traces, metrics and logs; GenAI
  dialect generations and OpenInference mapping; Claude Code plugin plus
  Agent Skills packaging that teaches agents when to render what; docs,
  examples, crates.io readiness; A2UI emitter spike behind the
  protocol-agnostic scene (watch item, not a commitment).

## Decisions proposed for lock (flagship defaults)

1. Owner david-engelmann, public repo, name from David's pick (availability
   on GitHub and crates.io verified before creation; the chronicle-cli
   lesson).
2. Licence MIT OR Apache-2.0.
3. Rust 2024 edition, latest stable pinned via rust-toolchain.toml,
   Cargo.lock committed.
4. Dependency discipline per the fleet rule: core crates dependency-light;
   named exceptions are serde (spec), fontdb and harfrust (text), petgraph
   (graph model); HTTP and MCP machinery lives in the server crate only.
5. Determinism law: byte-identical output for identical spec, data and
   engine version; golden-file corpus gates every PR.
6. Realistic-data testing: OTel Demo traces and sanitized recorded Claude
   Code and Codex session telemetry as fixtures; no private data in the repo.
7. docs/INVARIANTS.md with one-to-one invariant traceability; DECISIONS.md
   immutable inside the seat once locked; deviations escalate to Main Muse.
8. Seed and sweep discipline adapted: the golden corpus plus property tests
   over the derivations (conservation, bucketing, critical path) on every PR.
9. Milestone gates audited by Main Muse in the flagship pattern; merging is
   the owner's act alone.

## Open risks carried honestly

Sugiyama quality is the largest single work item if in-house layout is
needed. Host support for MCP Apps is young and uneven; the fallback ladder
is a launch requirement. Per-host size caps for UI resources are UNVERIFIED;
the engine designs to a self-imposed budget (about 150 KB for a typical
inline embed) and aggregates server-side beyond it. CJK and Arabic service
names outrun a layout that budgets one column per character. Rounds 31 to
34 closed the width half of this risk under INV-92, with layout width
derived from the Unicode Character Database. Text shaping and
bidirectional ordering stay outside the renderer by the Round 31 design's
fence. GenAI semconv
churn continues; attribute mapping stays isolated in one module and draft
names never enter the spec's public surface.
