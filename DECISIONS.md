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

## Landing record, 2026-11-17. M1 closed, M2 opened

M1 landed on main at 57fc9732054dee7f88e3df090ae7a617609c9282, and the
hosted gate passed on that exact head (Actions run 44006217699: format
clean, clippy clean with warnings denied, 37 tests passed and none
failed). Main Muse audited the gate independently against the remote,
recomputed the golden render claims from the fixture, and approved M1.

Two Minors from that audit are carried into M2 as hardening work, and
the M2 opening pass lands them. INV-1 and INV-9 shared one enforcing
test; they are split into one test each. INV-21 rested on the golden
corpus; it gains a direct assertion test on the palette assignment.
The property tests gain a shrinker, so a failing seed reports a minimal
span set alongside the seed instead of the seed alone.

M2 opens as scoped in the milestone list above: the field system,
service map with RED table, flame from profiles with labelled span
fallback, latency heatmap with exemplar overlay, and the agent-run
model with the Claude Code dialect adapter and logs-first fusion. The
live observability addendum stays a milestone of its own after the
static core and is not pulled into M2.

## Landing record, 2026-11-17. M2 closed

M2 landed on main at 3259c3a82e704884b76fd731dedb592790130a22, and the
hosted gate passed on that exact head (Actions run 44016612335: format
clean, clippy clean with warnings denied, 61 tests passed and none
failed). The slices landed in order: the field system (INV-24 to
INV-26), the latency heatmap with exemplar overlay (INV-27 to INV-29),
flame graphs from folded-stack profiles with the declared-count rule
and the labelled span fallback (INV-30 and INV-31), the service map
with evidenced edges and the ranked RED legend table (INV-32 and
INV-33), the canonical agent-run model with the Claude Code adapter
and its fusion rules (INV-34 and INV-35), and the CLI plumbing, under
which render takes profiles and session exports and session prints a
run as JSON. The two M1 Minors closed in the opening pass, as the M1
landing record above describes.

Review state, stated plainly: the checkpoint reviews were the seat's
own re-reads at each slice, and every golden was read back against its
fixture by hand before it landed; no independent review lanes ran.
No local full-suite run exists, because this box still cannot run
cargo; the hosted gate on the exact head is the merge evidence, as it
was for M1. One slice-5 correction is worth the record: the span
fallback flame's total is summed self time, not wall-clock duration,
because sibling spans overlap in time, and the render says so.

The live observability addendum stays a milestone of its own after the
static core and was not pulled into M2. M3 is next on the milestone
list: the MCP server with the triple output contract, the MCP Apps
view, the Codex dialect adapter, and HTML export.

## Gate verdict, 2026-11-17. M2 approved, M3 opened

Main Muse audited the M2 gate independently and approved it. The
audit verified both green runs on their exact heads with identical
code under them, the 61-test inventory matched per crate, the
53-blob tree with all six goldens, both M1 goldens byte-identical at
the M2 head by blob SHA, both M1 Minor closures present in the test
files, and the heatmap band totals, the full flame percentage set,
and the agent-run session totals recomputed independently, all
matching.

Two Minors and one nit carry into M3 as hardening, and the M3
opening pass lands them. INV-30's declared-count mismatch path was
implemented but untested; it gains a dedicated render-error test.
INV-31's fallback note and INV-33's layout rested on the golden
corpus; each gains a dedicated assertion test. INVARIANTS.md listed
its M2 sections out of numeric order; the sections now run in
invariant number order end to end.

M3 opens as scoped in the milestone list above: the MCP server with
the triple output contract (concise text, compact structured
content, and the view), the MCP Apps view, the Codex dialect adapter
with renderer-side token pricing, and standalone HTML export for
terminal hosts. The live observability addendum stays fenced as its
own milestone after the static core.

## Landing record, 2026-11-17. M3 closed

M3 landed on main at bd696474541eb039995f197a08e6791d78d2410b, and
the hosted gate passed on that exact head (Actions run 44021182295:
format clean, clippy clean with warnings denied, 80 tests passed and
none failed). The invariants and README records followed at
749e4b1f, also green (run 44021469386). The slices landed in order:
the opening pass, which recorded the M2 verdict, put INVARIANTS.md
in numeric order, and gave INV-30, INV-31, and INV-33 the dedicated
assertion tests the M2 audit asked for; the Codex dialect adapter
(INV-36 to INV-38), which shares the OTLP log reader with the Claude
Code adapter, normalizes gross input tokens to non-cache input, and
prices tokens from a table the renderer owns, dated 2026-11-16,
covering the gpt-5.1 and gpt-5.1-codex families at their published
rates, with unpriced models named and their cost excluded; the
standalone HTML export (INV-39); and the MCP server with the MCP
Apps view (INV-40 to INV-42), a new telorama-mcp crate whose render
tool returns concise text, compact structured figures from the same
derive functions the renderer uses, and the SVG in _meta for the
view, with an app-only drill-down tool for panel detail.

Review state, stated plainly: the checkpoint reviews were the seat's
own re-reads at each slice, and every golden was read back against
its fixture by hand before it landed; no independent review lanes
ran. No local full-suite run exists, because this box still cannot
run cargo; the hosted gate on the exact head is the merge evidence,
as it was for M1 and M2. The shared GitHub rate limit reached zero
once during the Codex golden generation; the seat stopped hard and
resumed after the reset, per standing discipline.

The live observability addendum stays a milestone of its own after
the static core and was not pulled into M3. M4 is next on the
milestone list: incident composition, GenAI and OpenInference
mapping, plugin and Agent Skills packaging, and crates.io readiness.
