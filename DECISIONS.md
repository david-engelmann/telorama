# DECISIONS.md

Locked 2026-11-16 by David Engelmann in the main chat. The name is diorama.
David's words were "I love diorama, lock it in." This file is immutable
inside this seat from the moment of the lock. Any deviation escalates to
Main Muse. The design record behind these decisions is docs/DOSSIER.md.

Supersession, 2026-11-16 evening. The name was renamed on David's
instruction the same evening, to give the repo the best crate name. The
bare name diorama is taken on crates.io by an unrelated crate, so Main
Muse chose telorama, telemetry plus the diorama suffix, as one name for
the repo, the crate, the binary, and the MCP server. Main Muse verified
firsthand that crates.io returned 404 for telorama and the telorama-
prefixed family and that david-engelmann/telorama was free on GitHub.
This seat re-verified both firsthand before creating anything. The
seat's census, 2026-11-16: crates.io returned 404 for telorama,
telorama-core, telorama-scene, telorama-ingest, telorama-svg,
telorama-mcp and telorama-cli, and david-engelmann/telorama was created
public the same evening (repo id 1460802971). Decision 1
and the crate names in the architecture section are read with this
supersession applied. Every other decision stands exactly as locked.

## Locked decisions

1. Owner david-engelmann, public repo, name telorama (locked as diorama,
   renamed the same evening under the supersession above). Availability
   on GitHub and crates.io verified before creation. The GitHub name
   david-engelmann/telorama was free at creation. On crates.io the bare
   name telorama is free, so the command line crate and the binary carry
   the bare name, and the library crates carry the telorama- prefix. The
   full crate name census is part of M1 publication readiness and is
   confirmed again before any publish step.
2. Licence MIT OR Apache-2.0.
3. Rust 2024 edition, latest stable pinned via rust-toolchain.toml,
   Cargo.lock committed.
4. Dependency discipline per the fleet rule. Core crates stay
   dependency-light. Named exceptions are serde for the spec, fontdb and
   harfrust for text, and petgraph for the graph model. HTTP and MCP
   machinery lives in the server crate only.
5. Determinism law. Byte-identical output for identical spec, data and
   engine version. The golden-file corpus gates every PR.
6. Realistic-data testing. OTel Demo traces and sanitized recorded Claude
   Code and Codex session telemetry serve as fixtures. No private data in
   the repo.
7. docs/INVARIANTS.md carries one-to-one invariant traceability.
   DECISIONS.md is immutable inside the seat once locked. Deviations
   escalate to Main Muse.
8. Seed and sweep discipline adapted. The golden corpus plus property tests
   over the derivations (conservation, bucketing, critical path) run on
   every PR.
9. Milestone gates are audited by Main Muse in the flagship pattern.
   Merging at gate points is the owner's act alone.

## Architecture carried from the dossier

Pipeline, all Rust, one direction. SceneSpec (JSON, agent-authored,
versioned) goes through validate and canonicalize, then derive (pure
functions), then layout, then a DisplayList with all text pre-measured and
positioned, content-hash element IDs and canonical float formatting, then
the emitters.

Crates named in the dossier are scene-core, ingest, layout, text, svg,
html and mcp. In this repo the published library crate names carry the
telorama- prefix, and the command line crate carries the bare name. M1
opens with telorama-scene (spec, schema, honesty validator and
derivations), telorama-ingest (OTLP file ingest for traces), telorama-svg
(DisplayList and the deterministic SVG emitter) and telorama (the command
line front end, binary name telorama). Later milestones add the remaining
crates under the same naming rule.

Honesty rules are enforced by the validator, not by prompt discipline.
Durations are percentiles, never averages. Sankey flow is conserved with
an explicit residual bucket. Heatmaps render real histogram buckets with
no interpolated surface. Flame graphs declare sample counts and refuse
below a floor. Exemplars that were never captured degrade to a window
search, never a fake link. Every panel carries source, exact query, window
and sampling state. Derived numbers show their derivation.

Scope fences. No datastore. No query language. No dashboard editor, no
alerting. No hosted renderer. Rendering is local and in-process with no
network in the render path. Generic business charts appear only as
supporting panels inside telemetry scenes. The scene spec stays small and
semantic. Agents never supply geometry, decimals or CSS.

## Milestones carried from the dossier

- M1. Scene spec v0 with schema and honesty validator. OTLP file ingest
  for traces. Waterfall plus stat and RED panels to deterministic SVG.
  Golden corpus from realistic fixtures (OTel Demo traces).
- M2. The field system (units, thresholds, legend tables, shared cursor).
  Service map with RED table. Flame from profiles with labelled span
  fallback. Latency heatmap with exemplar overlay. Agent-run model with
  the Claude Code dialect adapter, logs-first fusion.
- M3. MCP server with the triple output contract. MCP Apps view (shell
  plus scene player) verified rendered in Claude. Drill-down through
  app-only tools. Codex dialect adapter with renderer-side token pricing.
  Standalone HTML export for terminal hosts.
- M4. Incident scene composition across traces, metrics and logs. GenAI
  dialect generations and OpenInference mapping. Claude Code plugin plus
  Agent Skills packaging. Docs, examples, crates.io readiness. A2UI
  emitter spike behind the protocol-agnostic scene (watch item, not a
  commitment).

## Addendum, 2026-11-16 evening. Live observability

David's direction, relayed by Main Muse as a post-M1 milestone
addendum. The full text is banked in the research record as
live-observability-vision.md. In substance: one scene serves two readers,
a human investigator and an agent investigator, from the same derived
truths. Scenes carry a ranked investigation layer, look here first, each
marker with its evidence attached. Live mode is a stream of complete
deterministic frames; frames replace, values never transition, and house
law holds on live surfaces. An agent-state panel family reports what the
working agent is doing and why, with rationale required by validation, so
the tools force the better behavior they display. This scopes as its own
milestone after the static core. It changes nothing in M1, and the
dossier fences hold unchanged.
