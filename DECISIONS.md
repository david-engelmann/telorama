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

## Gate verdict, 2026-11-17. M3 approved, M4 opened

Main Muse audited the M3 gate independently and approved it. The
audit verified both green runs on their exact heads with identical
code under them, the 80-test inventory matched per suite, the
62-blob tree with all seven goldens, both M2 Minors discharged as
dedicated tests and the section-order nit fixed, the Codex turn
costs and session total recomputed exactly from the fixture and the
in-code pricing table, the MCP _meta SVG proven byte-equal to the
golden by direct string equality, and the README codepoint scan
clean.

One Minor carries into M4 as hardening. The unpriced-model
disclosure is proven at the run level by the ingest test, but no
golden exercises the priced and unpriced split as printed output;
M4 adds an unpriced-model golden.

M4 opens as scoped in the milestone list above: incident scene
composition across traces, metrics, and logs; GenAI dialect
generations and OpenInference mapping; Claude Code plugin plus
Agent Skills packaging; docs, examples, and crates.io readiness;
and the A2UI emitter spike behind its watch-item fence. The live
observability addendum stays fenced as its own milestone after the
static core.

## Landing record, 2026-11-17. M4 closed

M4 landed on code head a4e43776, with the hosted gate run
44066173393 green on that exact head: format, clippy, the full
test suite (95 tests passed, 0 failed), and the new Package step
all succeeded. The gate's own commit history for the milestone
stands in the repository log.

What landed, in the order it landed. The carried Minor from the
M3 verdict is discharged: the unpriced-model golden renders a
Codex session with one priced and one unpriced model, and the
split is printed on the face of the scene (golden landed at
3c033dea, gate run 44023018540). GenAI and OpenInference span
dialects map into the canonical agent-run model through one tree
walk (INV-43 to INV-45): the GenAI adapter reads the union of the
model and cache attribute shapes across the 1.30, 1.37, and 1.41
generations, both span dialects normalize gross input to
non-cache input and price from the shared table Codex uses, and
the CLI and MCP session paths take both dialects. The slice
exposed a real ingest bug, OTLP JSON carrying intValue as a
quoted string straight into span attributes, which silently
zeroed numeric attributes; the parser fix is general and the
first generated golden is what caught it. Incident composition
(INV-46, INV-47) is a pure template in telorama-scene with fixed,
stated choice rules, exposed as a library function, a CLI
incident command, and an incident_scene MCP tool on the triple
contract; the composed checkout scene is the tenth golden, its
figures verified by hand against the fixture before landing.
Plugin and Agent Skills packaging (INV-48) adds the plugin
directory with the Claude Code manifest, the MCP registration,
and the telorama skill. Docs, examples, and crates.io readiness
complete the milestone: three worked examples against the repo
fixtures, full package metadata on all five crates, a hosted
Package step that packages every crate in dependency order
through a staged directory source, and the A2UI emitter scoped
behind its watch-item fence in docs/A2UI-SPIKE.md, a question
with a stated trigger rather than a commitment. The publish
itself remains the owner's act, with the token supplied through
secure entry only.

Review state, stated plainly: the checkpoint reviews were the
seat's own re-reads at each slice, and every golden was read back
against its fixture by hand before it landed; no independent
review lanes ran. No local full-suite run exists, because this
box still cannot run cargo; the hosted gate on the exact head is
the merge evidence, as it was for M1, M2, and M3. The shared
GitHub rate limit reached zero more than once during the
milestone; the seat stopped hard each time and resumed after
the resets, per standing discipline. The box write fault refused
exec-identity writes through the milestone's second half, so the
incident golden's local copy was assembled through the file
tools in line-aligned chunks and verified against the remote
blob before the fleet-records mirror ran.

The static core is complete at M4. The live observability
addendum stays a milestone of its own after it, fenced as
recorded in the addendum above, and opens only on direction
from Main Muse.

## Milestone opening, 2026-11-19. M5, live observability

Main Muse directed the fence lifted on 2026-11-19, on David's
full press order after the credit reset. M5 opens as the live
observability milestone, scoped from the terms of
docs/LIVE-VISION.md and designed in docs/LIVE-DESIGN.md, which
is the milestone's design record. The design was written before
the build, with invariants INV-49 through INV-57 drafted in it
in advance, under the same discipline as the static core. Five
slices are planned. The investigation layer first, then the
agent-state panel, then frames, then the view stepper, then
close-out. The fences from the vision hold. No datastore, no
query language, no dashboard editor, no alerting, no hosted
renderer, no streaming server. The crates.io publish remains
the owner's act and is not part of this milestone.

## Landing record, 2026-11-19. M5 closed

The live observability milestone is landed. The design record
landed as PR 2 (main a0c29c30), the investigation layer as PR 3
(b920a770), the agent-state panel as PR 4 (cd99fe5c), frames as
PR 5 (532110bf), the view stepper as PR 6 (281334ea), and the
checkpoint repairs with the close-out records as PR 7. Main
stands at f44f885f, its tree verified identical to the final PR
head, and the closing gate is run 44637518364, SUCCESS, with
117 tests, up from 95 at M4. The golden corpus stands at twelve
scenes. The fleet-records mirror carries the milestone in the
same passes, verified blob-for-blob.

The checkpoint ran the standing three reviews. The red team and
the standards axis each returned two Majors, the spec axis
three Minors, and every finding was repaired in PR 7 or
answered here. Frame purity is now exact on every coverage
axis: the log dialects count seen spans per turn by window, so
no frame can quote a figure from a later turn. The agent_state
panel's data entry must declare the agent_state kind, enforced
by the validator and the renderer both. One shared step
traversal carries the totals, pricing, and wait censuses. The
Claude Code carry and stray coverage rules and the Codex
pre-prompt residual gained their first tests.

Two refinements of the design record stand as built and are
recorded here as the milestone's own corrections. The standing
5 percent threshold evidence rides the errors finding, because
the threshold lives in the RED derivation; the design draft's
sentence had placed it under latency. Frame scenes speak only
of their own prefix, after the first headline naming the
session's total turn count failed the truncation test the
design itself required.

The crates.io publish remains the owner's act, unchanged.

## M5 spec follow-ups, 2026-11-19

The spec fidelity axis's full report reached the seat after
the landing record above and named four deviations the repair
pass had not covered. They are repaired in a follow-up pass on
the same day. A finding now carries its evidence numbers as
structured data beside its statement, as the design's finding
shape specifies. The cost finding names the costliest turn's
model in its statement and its evidence. The view shows its
stepper whenever a frame stream is delivered, a one-turn
session included. INV-57's entry now names the session tool's
findings attachment, which the build had extended past the
design's letter without recording it. The standing verdict
threshold the errors finding compares against is read from the
field system's own rule rather than a copied literal. Two
deviations the report named were already answered by the
repair pass it predated: frame coverage is a true prefix sum
on every axis, so the README's sentence stands as written, and
the threshold placement is recorded in the landing record
above. On the duration half of that sentence, the position is
this. The standing duration verdict judges a service's RED
duration, which the RED row itself prints under its own
threshold rule. The latency finding's subject is one span's
self time, which no standing verdict judges, so no crossing
sentence is added there.

## M5 red-team follow-ups, 2026-11-19

The red team's full report reached the seat after the spec
follow-ups and named one Major the repair pass had not
covered. The Claude Code fusion counted merged spans per
application, so one blocked child pricing two steps in its
turn counted twice in the turn and once in the run. Merged
spans are now counted per turn as a set of span ids, the run
total is the union over kept turns, and a test with two steps
inside one waited tool span pins it. The same pass answers
the report's remaining Minors and Notes. A run whose turns
all drop away now reports zero coverage on every axis in both
log dialects, so the turn sums equal the run totals for every
run, not only runs with turns. The errors finding's exemplar
now comes from the same server-span basis RED counts on. A
blank waiting_on or next in an agent-state report normalizes
away at parse instead of rendering an empty line. The frame
stream's text says one turn when it means one turn. The
report's other items were already answered by the passes it
predated: frame coverage is a true prefix sum, the threshold
literal is read from the field system, the cost finding names
its model, and findings carry their evidence as data.

## Post-M5 perfection pass, 2026-11-21

The M4 audit carried one standing note out of that milestone:
INV-44's legacy GenAI cache attribute forms were verified in
the code but exercised by no fixture. That note is now
discharged. A legacy-shape fixture spells cache read and
cache creation with the pre-dotted attribute names
(gen_ai.usage.cache_read_tokens and
gen_ai.usage.cache_creation_tokens) and the provider with
gen_ai.system, and a dialect test pins the parsed run to the
same totals the dotted forms produce, with cache creation
counted in the turn totals and never priced. The showcase
media pass of the same period landed as PR 12: the README
and the worked examples now carry the frame-stepping GIF
and three stills, all of them the renderer's own output.

## crates.io publish, 2026-11-22

David approved the publish on 2026-11-22 and it is done.
All five crates stand on crates.io at version 0.1.0,
published in dependency order and each confirmed live
before the next was sent: telorama-scene, telorama-ingest,
telorama-svg, telorama, telorama-mcp. The stored crates.io
credential carried the publishes; the first publish doubled
as the auth test and no replacement token was needed. The
packaged files were built on a CI runner because this
machine refuses local binary writes, and the publish
metadata was derived from the packaged manifest inside
each crate file and checked dependency by dependency
before each upload. The README now records the published
state and the cargo install lines.

## Round 1, the sankey renderer, 2026-11-22

The roadmap's first round is landed. The sankey panel from
docs/SANKEY-DESIGN.md is built and the README no longer
carries the sentence that admitted the gap. Span flow is
staged by call depth and counted in spans, with orphans and
cycles entering through the panel's named residual bucket,
under invariants INV-58 to INV-61. The golden corpus stands
at thirteen scenes and the suite at 129 tests.

Two design points moved during the build, both recorded in
the design record itself. The first draft's conservation
claim failed on fan-out, a root with two children carries
more outflow than span count, so the accounting now states
the exact identities and the renderer draws each block at
the greater of its span count and its outflow. The
renderer's unsupported-panel guard retired in favor of
exhaustive matches, which clippy requires and which fail a
future panel type at compile time rather than at render
time.

The pre-landing reviews earned their keep. The red team
found that a repeated span id within a trace could panic
the derivation; the fix stages one canonical copy of the
id, chosen by the span's own fields alone, and stages every
other copy by its own parent chain, pinned by a determinism
test the first build would have failed. The same pass
corrected the residual caption, which had claimed a missing
parent for cycle entries, and narrowed deep-trace columns
past seven stages instead of letting them overlap. One
review point was dismissed with its reason: the MCP panel
summary keeps reporting counts, matching the service map
summary beside it, while the detail tool carries the full
node and link arrays. A new tool test pins both surfaces so
neither can drift silently.

## Round 2, the gallery, 2026-11-23

The gallery is landed. The design record went in first as
PR 17, then the build as PR 18. The page stands at
docs/gallery.html, one static file with no script in it,
assembled on a CI runner by its own freshness test running
under UPDATE_GALLERY and guarded by that test from then on.
Twelve sections cover the thirteen goldens, the incident
pair shown once with its findings strip, under invariants
INV-62 to INV-64. The README's first screen links the page
and the suite stands at 130 tests.

The checkpoint reviews changed the gate itself. The
freshness test as first built pinned the page's bytes but
not its coverage, so a future scene could have joined the
corpus while the gallery silently omitted it and INV-63
went false under a green suite. The test now enumerates the
fixture directory and requires every fixture scene's title
and question among the sections, with the incident
composition as the one extra. The same pass made the test
assert the README link its invariant claims, and made the
no-script check case-insensitive.

One wiring question took evidence from both tools. Session
scenes in the gallery follow the session wiring both tools
share, findings ranked over the run alone. The GenAI scene
is the exception, because its spans are the scene's
declared data and sit in the render's data, so the
investigation ranks them alongside the run, as the MCP
render path does with its input spans. The CLI session
invocation ranks the run alone even there, because its
session spans never reach the investigation. That is a real
inconsistency between the CLI and the server over the same
scene, recorded here as a known divergence for a later
round to settle. The gallery follows the render's data.

Two points were dismissed with their reasons. Two cards
carry the same title because two fixture scenes share it,
and the cards are told apart by their provenance questions.
Renaming a fixture to prettify the page would falsify the
corpus the page exists to show. And the freshness test
cannot catch a traces argument swapped between two sections
of the same shape, a known limit of the golden pattern the
design record adopts, accepted with the coverage gate above
standing guard over the likelier failures.

## Round 3, dialect and price-table currency, 2026-11-23

The currency pass is landed. The design record went in
first as PR 19, then the build as PR 20, under invariants
INV-65 to INV-68. The research pass read the current
published conventions on 2026-11-23 and found the GenAI
conventions standing at the released 1.41 text, with the
semantic-conventions repository's GenAI material moved to
its own home and still unreleased past that. Claude Code
and Codex changed nothing the adapters read.

One fix in the round was silent data loss. Cache creation
gained a current spelling, gen_ai.usage.cache_write.input_tokens,
which the adapter did not read, so a current emitter's
creation count silently landed at zero. The read now tries
the cache_write spelling first, then the dotted creation
spelling of the 1.37 and 1.41 generations, then the legacy
underscore form. Each spelling carries its own fixture
mirroring the same figures, and a fourth test pins the read
order itself. Cache creation stays counted and never
priced.

The new operation and kind mappings are invoke_workflow as
agent structure, generate_content as a model call, and the
OpenInference RERANKER kind as a tool step. The workflow
fixture carries a second trace nesting a workflow under an
agent, because at the root the classification is invisible
and the first build's test could not have caught it
regressing. A plan span in the same trace pins its
container behaviour by the tokens its child contributes.

Two decisions are recorded rather than built. A DECISION
span stays a transparent container, because the kind
exists to keep decision usage out of LLM token counts and
the canonical model has no decision step, and mapping it
to a model call would break the separation the kind
exists for. It reopens when the canonical model grows a
decision step. Span-carried cost attributes are never
read, because the renderer's table is the single pricing
law and a second source would fork the honesty rules.
Both decisions carry tests that would fail if the code
drifted from them.

Price table changelog, 2026-11-23. The table's date
advanced from 2026-11-16 to 2026-11-23 and it stands at
26 priced models. Every existing row was rechecked
against the publishers' pages and none had moved. New
rows are gpt-5.1-codex-max in its family, gpt-5.2,
gpt-5.2-codex and gpt-5.3-codex, gpt-5, gpt-5-codex,
gpt-5-mini, gpt-5-nano and codex-mini-latest, and the
first Claude rows, claude-sonnet-5, claude-sonnet-5.5,
claude-opus-5.5, claude-opus-5 with claude-opus-4.5
through 4.8, claude-sonnet-4.5 and 4.6, claude-haiku-4.5,
and claude-fable-5.2 with claude-mythos-5.2. Claude cache
read prices are explicit in every row, because Anthropic
no longer prices cache reads at a uniform share of input.
The gpt-5.1-mini row stays even though the current
pricing page no longer lists it under that name, because
traces in the wild name it and its figures are
corroborated by the identical gpt-5-mini row. The pro
rows are skipped for want of a published cached price,
claude-haiku-5.5 is skipped because its price is tiered
by prompt size and does not fit the table's flat row
shape, and the GPT-6.2 models stay unpriced because no
API price is published yet. The table's arithmetic is
pinned figure by figure, one token class at a time, so a
transposed figure now fails the pin as surely as an
edited one. Only the three priced goldens and the gallery
page moved, each by the disclosure date alone.

Two items are named for later rounds. Claude Code's
api_refusal event is the first candidate for the next
convention pass, held back here because its fields were
not verified from a live export and this programme builds
fixtures before support. Normalizing dated snapshot model
ids before pricing is a named open question, deferred
because it changes pricing behaviour and needs its own
fixture-backed design. The CLI session findings divergence
the gallery round found stands open and is not closed by
this round.

The checkpoint reviews found no defect in the dialect or
pricing code. Their findings were enforcement gaps, each
closed in the round. The workflow classification pin, the
dotted spelling fixture, the read-order pin, the DECISION
and span-cost pins, and the per-figure price pins above
all came out of that pass. One standards note needed no
action, the price match's first arm taking block form
under rustfmt while its neighbours keep expression form.

## Round 4, scale and performance proof, 2026-11-23

The scale round landed under docs/SCALE-DESIGN.md with
invariants INV-69 through INV-73. The fixtures are
generated inside the tests from fixed arithmetic. Ten
thousand spans in one deep trace, a wide trace of ten
thousand spans at depth two, ten thousand spans across
one thousand traces, five hundred session turns, and one
hundred thousand stack samples. Every fixture pins its
renders by FNV-1a hash and its headline counts by
assertion, and every scale test carries a wall-clock
ceiling at ten times its first measured hosted figure
under INV-70. The suite stands at 150 tests.

The round convicted one suspect and cleared one. The
sankey staging walk rebuilt its per-trace span index by
filtering the whole canonical map once per trace, and
the thousand-trace fixture measured 717 milliseconds
against the single trace fixture's 114 at the same span
count. The fix groups each trace's canonical spans once
in the canonical pass, and the fixed code measures 82
milliseconds on the same fixture. The frame stream's
prefix cloning was cleared with evidence. Five hundred
frames build in 97 milliseconds, and the rendered
stream totals 104,428,401 bytes, held under INV-71 as
the quadratic bound the design states.

One defect the suspect list did not name was a panic.
The service map's label budget reached zero on the
trace fixture's cyclic service graph and the truncation
helper subtracted from zero. The helper is now total at
a zero budget, the service map floors its box width and
label budget the way the sankey narrows past seven
stages, and INV-73 states the rule. The layering itself
still relaxes around a cycle up to the node-count bound.
That is recorded in the design record as an observation,
and a cycle-aware layering is design work for a later
round. A neighbouring hazard is named here on the red
team's evidence. The waterfall's critical path walk has
no cycle guard, so a cyclic trace rendered as a
waterfall would hang rather than error. It predates this
round, the scale fixtures are acyclic, and no fix is
made here.

The flame renderer holds a standing bound of 48 levels
on flame trees, set before this round. The deep
fixture's span tree stands a thousand levels past it, so
the flame render is proven on the wide fixture at the
same span count and the deep trace's refusal is pinned
as a named error. The design record's amendments carry
the full account.

The checkpoint reviews ran on the pre-repair head and
found the code fixes correct and output-preserving. The
grouping change was verified identical in construction
by both lanes. Their findings were that the paper trail
claimed renders the build had not produced, that the
many-trace ceiling was set from the convicted code's
measurement and so could never catch that code
returning, that the per-node conservation identity was
checked only in aggregate, and that the zero budget
guard had no test through its one live caller. Every
finding was closed in the repair pass, and the repairs
were verified by the hosted gate on the final head,
which names each new and strengthened test. Two
informational notes needed no action. The duplicated
scale recipes across the scene and svg test files are
forced by the per-file test binary structure and match
the repo's established pattern, and the measurement
lines beside the ceiling asserts are the re-measurement
instrument the ceiling method depends on.

One process incident is owned plainly. A scripted edit
through the shell truncated the derive test file to zero
bytes mid-round, against the house rule that edits go
through the file tools. The file was restored byte-exact
from the pushed branch blob and verified by hash before
work continued. No content was lost.
## Round 5, the dark theme (2026-11-23)

Design record docs/THEME-DESIGN.md with INV-74 to
INV-77. The scene spec gains an optional theme field,
light by default, and the renderer resolves every
color it chooses through one palette record per
theme. Colors a scene declares for itself are data
and pass through unchanged in both themes.

One sentence in the design record was wrong and the
record now carries a dated amendment. A bare SVG
does paint a background rectangle, a full-canvas
ground the display list has emitted since the first
renderer. The theme governs that rectangle too, and
INV-77 is worded to match. The amendment also
clarifies that the HTML export paints the palette
background in both themes, the same ground the
drawing carries.

The contrast floors of INV-75 are asserted for the
dark palette only, by the invariant's own wording.
The light theme's bytes cannot change under INV-74,
and the light palette carries one pre-existing pair
under the text floor, the tool-step green on white at
about 3.4 to 1, which no change this round may touch
without breaking the light regression pin. INV-75
also names the grid strokes as chrome with a quiet
floor of their own, reconciling the invariant with
the test at the checkpoint reviews' direction. The
light grid has always sat at the same subtlety.

The checkpoint reviews caught three defects, all
repaired in this landing. The dark label rule for
text drawn on a fill first read as the ground color
for every fill, which left the flame graph's root
label, drawn on the surface fill, at 1.12 to 1
contrast. The rule now picks whichever of the ground
and the ink contrasts more with the fill, the
contrast test asserts that best pair for every fill
a label can land on, and a dark flame golden pins
the bytes. The service map's missing-service node
fallback had been mapped to the palette fallback
field, whose light value differs from the literal
it replaced. It now maps to the residual field,
whose light value is the original literal, and the
path stays unreachable on current call paths either
way. The normalization test now covers the agent
run, the incident composition, and the agent state
scenes alongside the span scenes, so every panel
renderer stands under a dark proof, a normalization
proof, or a dark golden.

Two review judgements were considered and kept as
built, with reasons. The palette record carries a
dark flag so the label rule can branch. A single
contrast rule for both themes was rejected because
it changes a light byte, the tool-step green label,
whose light choice the luminance rule makes and the
contrast rule would reverse. The unthemed
standalone_html stays as published API, mirroring
the delegate pattern the frame and incident builders
use. The name_hash helper now takes the palette
length rather than the record, and the frame
builder's dark path has its own test.

The CLI session-findings divergence recorded in the
Round 2 entry stands open after this round. It is
not a theme concern and this round does not touch
the findings wiring.

## Applicability charges from the standards audit (2026-11-23)

Three charges from the ontology and standards audit
landed as records, each audited firsthand against
the code before any of them was written down.

The GenAI dialect reference is re-pinned. The
conventions now live in their own repository,
open-telemetry/semantic-conventions-genai, which
has cut no versioned conventions release, and every
stability marker in it still reads Development. The
adapter therefore pins to a named commit,
ae8214a8f3a98786071736855093b74621174dc4 of
2026-11-23, read against the released 1.41 text,
and the dialect module's own doc carries the same
pin. The repository does carry one tag, v0.1.0, and
the record states plainly that the tag is a chore
release of the repository itself and not a
conventions release, which is why the commit and
not the tag is the pin. That corrects the audit
report's claim that the repository carries no tags.

The provider attribute was audited and needs no
code. Neither gen_ai.provider.name nor its
predecessor gen_ai.system is read anywhere, because
the canonical agent-run model carries no provider
fact for a read to land in. Pricing and display key
on the model alone, and both spellings already sit
in the fixtures as inert attributes. The standing
rule is recorded here for the day a provider fact
is ever added to the model. It reads
gen_ai.provider.name first with gen_ai.system as
fallback, the cache-spelling union pattern the
adapter already uses. The removed
gen_ai.prompt.variable was audited in the same pass
and has zero references anywhere in the repo.

The MCP view was checked against the stable MCP
Apps contract, SEP-1865 as of 2026-01-26, and
conforms. The resource URI is ui://telorama/scene,
the mime type is text/html;profile=mcp-app, the
tools link the view through _meta.ui.resourceUri,
one tool carries _meta.ui.visibility of app only,
and the view runs the ui/initialize handshake and
reads its results on ui/notifications/tool-result.
No draft-only feature is relied on. The extra meta
keys the server sets, telorama/svg and
telorama/frames, are vendor-namespaced extensions
the spec permits. MCP Events, SEP-3415, stays a
watch item with no work taken.

## Round 6, the registry finish (2026-11-23)

Design record docs/REGISTRY-DESIGN.md with INV-78
and INV-79. The recon that opened the round read
the live registry pages firsthand and found the
0.1.0 release's readme media broken on both
registries. All five crates shared the repo root
readme, and crates.io and docs.rs each rewrite a
relative image path against the crate directory,
where the file does not live. Every crate page
showed two broken images.

Each crate now ships its own readme, short and
scoped to that crate, with media as absolute URLs
to the raw files on the main branch, and each
manifest names its own readme. The root readme
keeps its relative media for GitHub and its current
state section now matches the finished state,
including the dark theme and the corpus counted as
it stands. A packaging test pins the shape, the
version agreement across the five manifests and
the workspace pins, the readme each manifest
names, and the absence of relative link targets in
any crate readme.

All five crates move to 0.2.0 together. The changes
since 0.1.0 are features, so the bump is a minor
one. Each library crate gains one worked example in
its front-door documentation, run as a doc test by
the hosted gate. Two of the examples taught the
round something worth keeping. The ingest parser
refuses an export with no spans in it, so its
example parses a real one-span export, and the
honesty validator requires a headline on every
scene, so both example scenes carry one.

The version bump exposed a latent defect in the CI
package step, which named the 0.1.0 crate file
literally and failed after every test had passed.
The step now reads the version from the crate's own
manifest, which is exact for every crate and stays
right at the next bump. A first attempt at the fix
globbed the package directory and matched every
telorama-prefixed crate at once, which the same
gate caught before it could land.

The checkpoint reviews strengthened the packaging
test in two places. Its readme title check now
matches the first line exactly, so one crate's
title can no longer pass on a sibling's longer
heading, and its link scan now also covers
reference-style definitions and image sources, the
two relative forms the inline scan could not see.
Two stale sentences in crate doc headers, dating
from the first milestone, were swept in the same
pass. Two observations were dismissed with reasons
recorded in the gate report. The rewritten current
state paragraph does not name the gallery because
the readme's first screen already carries its link,
and the packaging test's two assertions beyond the
design's three enforce other sentences of the same
record.

One gap the reviews surfaced stands open by name.
A missing-docs deny was tried on the four library
crates in the repair pass and convicted 176
undocumented public items, nearly all of them
struct fields and enum variants in telorama-scene.
The deny was withdrawn. Documenting the public
surface field by field is a pass of its own, named
here as future work, and the round's documentation
claim rests on the worked examples and the
firsthand docs.rs read that follows the release.

## Round 6 publish record (2026-11-23)

All five crates are published on crates.io at
0.2.0, sent in the settled order, scene first and
the MCP server last, each confirmed live at 0.2.0
before the next was sent. The rendered registry
readmes were read back firsthand after the release.
Each crate page now shows its own readme, and the
four images resolve to the media files on the main
branch. The 0.1.0 pages' broken images are gone.

Packaging ran on a CI runner from the merged main.
Cargo would not assemble the packages while the
workspace dependencies at 0.2.0 were not yet on the
index, so the packaging workflow vendors the tree
and stages each package into the vendor directory
as it is built, the same loop the CI package step
runs. The publish itself used the stored credential
through the same driver as the 0.1.0 release.

The docs.rs builds for 0.2.0 were still queued when
this record was written. Reading those pages
firsthand stands open and closes the round's last
verification.

Completion note, the same day. The docs.rs builds
for 0.2.0 are live for all five crates, and each
library crate's front page carries its worked
example. The round's last verification is closed.

## Round 7: the documented surface (2026-11-23)

The public surface is documented field by field.
The Round 6 trial count of 176 undocumented items
was the first crate's alone, because the compiler
stops at the first failing crate; the gate on the
build convicted 63 more in telorama-svg and
telorama-ingest once telorama-scene compiled, and
the true total was 239 items, all now documented.
Each library crate root denies missing docs, so
the coverage is a permanent gate (INV-80). The
work was grounded per file, and the judgement
calls are recorded: Source.dialect and
Context.filter are documented as recorded
provenance because nothing in the codebase
consumes them, and Coverage.spans_seen is
documented as counted before any merging, from
the constructor note and the frame prefix sums.
Six pre-existing doc lines that broke the copy
bar were swept in the same pass, four in spec.rs
and two in telorama-svg.

Findings parity is closed (INV-81). parse_session
returns the session's spans alongside the run,
and the CLI render and incident paths rank
findings over the trace spans plus the session
spans, the inputs the MCP render path ranks. The
panels still draw the trace spans alone. The
parity is pinned by a test over a GenAI session
whose one error lives only in its spans; the run
alone convicts no errors finding, and the CLI
strip carries the span-based one. This closes
the divergence recorded as open since Round 2.

The reviews added three records. First, the Spec
review found the gallery leg of INV-81
unimplemented in the build as first pushed: the
gallery's Claude session section ranked over the
run alone while its spans sat in scope, and its
module doc still described the divergence as
standing. The section now ranks over the session
spans plus the run, the page is regenerated, and
the doc states the uniform rule; the Codex
sections carry no span exports, so their span
set is empty and the rule holds there unchanged.
Second, the parity test substitutes for the
design record's letter: it pins the CLI against
the library investigation over the same inputs
rather than invoking the MCP server, because the
MCP session_run tool ranks the run alone by
design and the render path's ranking is the same
library call. Third, two edges are named rather
than fixed: passing the same export as both
--traces and --session-spans double-counts those
spans in the findings (the panels are
unaffected), and the CLI incident command
composes its scene from the trace spans alone
while the MCP incident tool composes over its
full input set, a composition asymmetry outside
INV-81's findings letter. The Standards review's
two craft notes, the duplicated session binding
block in the two commands and the inner spans
shadowing, are kept as built: each command in
that file already re-parses its own flags in the
same pattern, and the binding reads correctly in
context. Its one must-fix, a semicolon in a new
coverage doc, was repaired in the same pass.

One process incident, owned: a builder lane ran
a local cargo check against the standing rule
that CI is the compiler on this box, and the
write fault truncated the local Cargo.lock to
zero bytes. The local copy was already stale and
the remote blob is banked in the seat's mirror
note, so nothing was lost; the lane's briefs now
carry the no-local-cargo rule by name.

## Round 8: hostile inputs (2026-11-23)

Cycle safety is closed (INV-82). The waterfall
rows walk, the critical path, and the flame
builder each stop at the first repeated span id,
and service layers come from a new derivation,
service_layers, that computes longest paths over
the service graph's condensation, so services in
one strongly connected component share a layer.
The build found a fourth walk the design recon
missed. flame_from_spans built its tree by
unbounded recursion, so a cyclic trace overflowed
the stack instead of hanging. One scale hash
moved, with the reason pinned in the test. The
scale fixture's services form a ring along the
chain, so its old service map hash had captured
the pre-INV-82 layering that a relaxation
inflates on a cyclic graph, and the new hash
pins the collapsed layout. The waterfall and
sankey hashes, and every golden, stand unchanged,
which is the acyclic byte-identity proof.

Refusals are visible (INV-83). The Claude Code
adapter reads claude_code.api_refusal events,
fixtures first, and each refusal becomes a model
call in its turn with the error flag set and zero
tokens, the shape an api_error call already gets.
The design record and the invariant correct an
overclaim in their first text. The errors ranking
is span-based, and failed calls surface in the
step list and the error drawing, not the ranking.

Snapshot ids price by their base model (INV-84).
The price lookup tries the exact id, then the id
with one trailing date suffix stripped in either
the YYYYMMDD or the YYYY-MM-DD shape, then the
dash version form read as the table's dot form,
and every candidate must match a table row
exactly. An id that matches nothing stays
unpriced and is reported as such. The suffix
rule reads digit shapes, not calendar dates,
and the reviews found its first form could
panic on a non-ASCII model id whose bytes put
a multi-byte character at the cut point. The
cut is now made only where the byte before it
is an ASCII dash, with a test pinning it.

The reviews added the rest of this record. The
guards act on repeated span ids, which is the
mechanism a cycle takes in this codebase,
because the children maps are keyed by span id.
A trace with duplicate ids and no cycle also
emits each id once, where the pre-round code
emitted every occurrence. The golden corpus
carries no duplicate-id fixture, so INV-82
states its byte-identity claim with the
unique-id qualifier. The red team broke the
round's first form in the span dialect session
builder, whose subtree count and step emitter
walked the same children map unguarded. Both
now apply the emit-once rule, pinned by a
cyclic session fixture, and INV-82's
enforcement line names the dialect tests.
service_layers indexes the graph's edges
against its nodes directly, which a hand-built
graph with an edge naming an absent service
could panic. service_graph cannot produce such
a graph, and the exposure matches the code it
replaced, so it stands as a named leftover. A
pre-existing limit also stands named. The flame
builder's recursion is bounded by distinct span
ids, and a very deep acyclic trace can still
overflow the stack, as it could before this
round.

## Round 9: parity completion (2026-11-23)

The two asymmetries the Round 7 reviews named
are closed (INV-85). Each CLI command that
takes traces and a session builds one combined
span set through a single helper, trace spans
first, with each trace id and span id entering
once and the first occurrence kept. The
incident command composes its scene over that
set and renders it from the same set, the
pairing the MCP incident tool has always used,
and both commands rank their findings over it.
The render command's panels still draw the
trace spans alone, as Round 7 settled.

The reviews qualified the round's claims in
one place, and the design record's amendments
carry the correction. The combined set dedupes
across the whole input, so a traces file with
duplicate span ids inside it changes output
against the pre-round code, where the fence
had claimed byte-identity for single-input
calls without the duplicate-free qualifier.
The same class now separates the CLI from the
MCP incident tool, which does not dedupe its
input. The two agree exactly on duplicate-free
input, and on malformed input the CLI reports
the single count. Widening the dedup to the
MCP tool is named as a candidate for a later
round, not taken here. The composition test
pins the combined window, which is the
observable the scene JSON carries. The design
record no longer claims a panel-level pin for
it. The first-occurrence rule is positional,
traces before session, which the record now
distinguishes from the sankey staging's
field-chosen canonical copy.

## Round 10: the named leftovers (2026-11-23)

The three leftovers Rounds 8 and 9 recorded are
closed (INV-86). The MCP input assembly dedupes
its parsed spans by trace id and span id, first
occurrence kept, the CLI combined set's rule, so
the tools that compose, rank, and render report
the same counts as the CLI on the same export.
service_layers builds its adjacency and its
component edges only from edges whose endpoints
both stand in the graph's nodes, so a hand-built
graph layers its named services instead of
panicking or absorbing an unknown name.
flame_from_spans assembles its tree on an
explicit stack with the same build-once rule and
insertion order, and a generated chain of fifty
thousand spans builds with its depth and root
value pinned.

The reviews widened the round in one place and
qualified it in three. The red team found the
renderer's depth verdict itself recursive, so a
deep flame panel overflowed inside the verdict
before the 48-level refusal could fire. The
verdict is iterative now, the renderer takes its
tree apart iteratively on every path out, and a
refusal pin stands at twenty thousand spans. The
record was corrected in the same round, per the
standing discipline. The design's claim that no
tool can opt out of the dedup is scoped to the
shared assembly, since the inspect tool parses
its own argument on both surfaces and reports
the raw counts on each. On duplicate-carrying
input the MCP render panels now follow the
deduped set while the CLI panels draw the raw
traces, a parting in the direction of the honest
count that the amendments name. The layers
test pins the panic through the component-edge
guard, the adjacency guard standing as
defense-in-depth for the same invariant. Two
depth paths are named as the next leftovers,
the MCP panel summary and detail paths, which
still own and drop the trees they build, and
flame_tree, the folded-stack builder, which
still inserts and freezes recursively.

## Round 11: depth, finished (2026-11-23)

The two depth paths Round 10 named are closed.
telorama-scene carries one public
dismantle_flame_tree, the renderer's private
helper is deleted in its favor, and the MCP
panel summary and detail paths dismantle the
flame trees they build once they have read
them. The folded-stack builder inserts by
walking the frame path in a loop and freezes
on an explicit stack. Both review lanes traced
the rewrites against the recursions they
replace on shared-prefix and branching
constructions and found the values, the node
membership, and the child order identical,
with the sort's total order making the output
certain twice over. INV-86's exception clause
is removed, the property it qualified now
holding without exception.

The red team's workspace sweep found the
finished property does not yet hold for
INV-87 as first written, and the record was
corrected in the same round. Two paths outside
the round's scope still recurse over input
depth. The dialect session builder's emit
recurses over an acyclic span tree, its Round
8 guard bounding cycles only, and the
agent-run layout walk recurses over a run's
nesting depth with no verdict capping it,
while the run trees drop recursively on the
same paths. INV-87 as landed names both as
the Round 12 leftovers, and the design record
carries the same correction in its amendments.
The MCP depth pin stands at fifty thousand
frames, raised from twenty on review, so the
panel detail dismantle is convicted by a test
at the depth where a drop is proven to
overflow. The panel summary dismantle, the
same one-line pattern in the adjacent arm, is
verified by reading.

## Round 12: the agent tree (2026-11-23)

The design record is docs/AGENT-TREE-DESIGN.md.
The build closes the two paths the Round 11 red
team named against INV-87. The dialect session
builder's emit runs on an explicit stack of
frames, each holding its own step sink. A
container frame splices its steps into its
parent on completion, an agent frame wraps its
sink in the subagent step, and the seen rule
fires at frame creation and leaf encounter, the
points the recursion tested it. The emitted
steps are the same steps in the same order, a
parity all three review lanes traced on mixed
trees, diamonds, and duplicates of frames still
open on the stack. The agent-run layout walk is
a pre-order stack walk through a row helper
whose body is the old function minus its
recursion tail, and the shared AgentStep walk
iterates on the same conversion.

MAX_AGENT_NESTING, 512, now lives in
telorama-scene. The dialect builder refuses a
session whose agent spans would nest past it
with a named error while building, and the
Claude Code adapter refuses the same way when a
subagent wrap would, measured by an iterative
nesting_depth over the captured steps. No
session is truncated, because a partial run
would misstate the session. The reviews swept
every SubagentRun construction in the
workspace. The Claude empty-run insertion needs
no check of its own, since it adds one level at
the top of a turn and any later wrap that
captures it passes the checked site, and the
Codex adapter builds no subagent steps at all.
The budget bounds the recursions that remain
over a run tree, its drop, its serialization,
and its derived clones, and the red team put
the margin at roughly four to ten times the
smallest stack in play, exercised by the
at-budget pins themselves.

Two pins land, one per enforcement site. The
GenAI pin builds a generated invoke_agent chain
whose steps nest one past the budget and
asserts the refusal names it, with a companion
chain at the budget building. The Claude pin,
added in the review repair pass, generates
agent spans whose windows nest around one model
step, smallest first, and asserts the same
two-sided result at the wrap. The budget check
tests the seen set before refusing, matching
the recursion's order, so a repeated agent span
past the budget is skipped rather than refused,
and it can form no step either way. INV-87's
exception clause is retired and its text states
the budget as the mechanism. The review repair
also split a semicolon out of the emit header
comment and corrected INV-87's remain-list,
which still named the shared step walk the same
diff had converted.

## Round 13: the registry, refreshed (2026-11-23)

The design record is docs/REGISTRY-REFRESH-DESIGN.md.
All five crates version together at 0.3.0, the
workspace pins move with them, and Cargo.lock is
regenerated in the packaging pass, its diff
exactly the five version lines. The root
readme's Current state paragraph is rewritten
for the finished surface. It counts thirteen
rounds, states the documented surface under its
deny gate, the cycle and depth law, the nesting
budget, and CLI and MCP findings parity, and
closes at version 0.3.0. Every new claim was
verified against the repo by the review lanes.

The red team's sweep found one live surface
still presenting an old version. The MCP
server's initialize reply hardcoded version
0.1.0, and the Apps view's ui/initialize named
the same, stale through the whole 0.2.0 era.
Both now read env!("CARGO_PKG_VERSION"), so the
handshake tracks the manifest forever, and two
assertions pin the served version against the
manifest in the MCP suite. This is a recorded
deviation from the design's no-other-changes
letter, taken because presenting the current
version honestly is the round's whole subject,
and the fix is confined to the two strings and
their pins. The repair's first form split the
view's raw string for a concat and silently
consumed the quotes around the version, which
the new pin caught on its first run. The quotes
stand restored as doubled delimiters.

The round count in the paragraph follows the
repo's own convention. The 0.2.0 paragraph
shipped in Round 6's build counting six rounds,
the round it shipped in included, so this
paragraph counts thirteen. The plugin
manifest's version stays 0.1.0 on its own
track, as it has through both registry bumps,
pinned deliberately by the packaging test. The
publish follows in this round's own record,
each version confirmed live before the next is
sent.

## Round 13 publish record (2026-11-23)

All five crates published at 0.3.0 on
2026-11-23 in the settled order, scene, ingest,
svg, telorama, telorama-mcp, each PUT answered
200 and each version confirmed live by an
independent read before the next was sent.
Every crate now lists 0.3.0 as its newest
version over 0.2.0 and 0.1.0. The rendered
registry readmes verify firsthand. The ingest
readme renders the agent run image, the svg
readme the incident image, the command line
readme the frames animation, and the MCP readme
the agent state image, every source the
absolute raw address the registry requires, and
the scene readme renders with no image, as
designed. The docs.rs pages for all five
crates serve 0.3.0, and the worked example
stands on the scene front page. The packaging
ran on a CI runner through the vendored staging
route, and the packaging branch is deleted.

## Round 14: the long session (2026-11-23)

The design record is docs/LONG-SESSION-DESIGN.md.
Three generated fixtures in the Round 4 style
prove the session side past the Round 4 point.
A five thousand turn run through the frame
stream builds in 13.06 seconds on the first
hosted measurement, under a ceiling of ten
times that, and its final frame renders at
4,150,019 bytes under a pinned content hash.
A ten thousand span GenAI session parses in 47
milliseconds and a ten thousand event Claude
Code log in 176 milliseconds, both asserting
turn counts and token totals. The adapters are
linear at this scale and need no ceiling
beyond the suite's own runtime.

The stream's aggregate byte total is the one
figure the gate does not materialize. At five
hundred turns the full render measured
104,428,401 bytes, and each frame grows by one
turn's rows, so the aggregate at five thousand
turns stands on the order of ten gigabytes by
that measured growth. The gate therefore pins
the stream's construction and its final frame,
and this record carries the aggregate as a
measured estimate rather than a pin. The
design's conviction branch does not fire. The
quadratic construction costs thirteen seconds
at five thousand turns, which is practical,
and the frames are built to be consumed one at
a time, as the view and the frames command
consume them.

The Claude fixture's first form put the event
name in the record body, where the parser does
not read it, and built zero turns. The parser
reads the event.name attribute, the shape the
real dialect carries, and the fixture now
builds its records with serde_json so the
shape cannot drift by a string template again.

The checkpoint reviews repaired the proof in
three places. The GenAI test's first coverage
assertion read the run-level figure, which the
builder copies from the input length, so it
proved nothing about emission. The test now
sums the per-turn coverage, the count of spans
the walk actually reached, and the Claude test
likewise asserts its event count. Both adapter
tests gained wall clock ceilings on the ten
times rule, as INV-70 asks of every scale test.
The frame test gained a mid-stream render pin,
so its proof is not only about the stream's
two ends. The design record carries a dated
amendment for the aggregate handling above.

The standing lanes stand as the design left
them. The A2UI watch was reviewed and stays a
watch item. The convention check rests on the
charges pass of this same day, which read the
GenAI conventions against the pinned commit
and the MCP Apps surface against the stable
text, and nothing in either text has moved
since that reading.

## Round 15: the second sweep (2026-11-23)

The round aimed the adversarial sweep at the boundaries the earlier rounds had not audited as a set, under the charter in docs/SWEEP-DESIGN.md. Three read-only audit lanes covered the text and markup surfaces, the parse and protocol surfaces, and the numeric boundaries with the remaining walks. The build fixes what the audit convicted, and INV-88 now states the law the fixes make true. The checkpoint reviews then broke two of the build's own claims and the repair pass closed both before landing, so the record below covers the repairs as part of the round. The hosted gate holds 207 tests, fourteen of them new pins, and the golden corpus and scale hashes stand unchanged.

The headline conviction was a crash. A Claude Code event whose cost_usd arrives as the string NaN, a well-formed OTLP shape, made the investigation's cost finding panic on a comparison NaN leaves undefined, in every build profile, on the automatic findings path every session render takes. A non-finite cost now reads as the defined default at the adapter, the finding itself refuses a non-finite total, and its leader comparison is a total order. The red team then showed the filter alone was not enough, because two finite costs near the f64 ceiling sum past it and the infinity reached the frame headline, the MCP session text, and the SVG total. Cost accumulation now goes through a shared add_costs helper in telorama-scene that keeps every running total finite, saturating at the largest finite value of its sign. A negative cost is a different case and is carried as a negative figure by design, since a refund-shaped event is representable in the source data. The same sweep found the framebook writing the telemetry-controlled session id into its HTML title and heading unescaped, the one hand-built document in the workspace, and the id is now escaped at both points. A script payload in a session id can no longer enter the book. The CLI keeps its own small escape helper rather than importing the SVG crate's, because the apostrophe entity that is safe in HTML content differs from the one XML defines, and the choice is recorded here as made rather than missed.

The arithmetic family was the widest. Timestamps reach the full i64 range through the parsers and the validator, and the subtraction and multiplication over them was unchecked in the span duration, the range seconds, the incident and frame window round-ups, the heatmap grid, the waterfall and agent-run windows, the investigation's sums and wall, and the log adapters' duration conversions, midpoints, and sort keys. In a debug build each overflow panics and in a release build each wraps. The family now computes in saturating forms throughout, so an extreme input yields a defined figure at the type's edge rather than a crash or a wrapped one. The red team's counter-sweep found five more sites of the same class that the audit lanes had not enumerated, in the self-time interval merge, the heatmap window inside the renderer, the agent-step detail duration, the permission-wait sum and its offset in the agent-run bar, and the MCP panel detail's start offsets, and the repair pass saturated those too, each with its pin. Negative token counts in the Claude and Codex adapters cast to figures near the u64 maximum and now read as zero through the same unsigned pattern the span dialects already used, and every token and sample summation saturates.

The parse boundary gained three refusals and alignments. An OTLP span carrying no traceId or spanId is refused by name instead of being aliased to an empty id. The two OTLP readers now agree, the logs reader accepting intValue as a string or a number and both readers skipping attribute shapes they do not model instead of one of them emptying the value. The MCP stdio loop answers a line that is not valid UTF-8 with a parse error and keeps serving, where it once exited in silence. A trace whose spans form a rootless cycle now draws a render error that names the cycle, where the waterfall and flame panels once reported that no spans were found.

Two audit findings were recorded without a code change. Scene duplicate keys collapse inside serde before the validator can see them, so a repeated key in a hand-written scene resolves last-wins by the parser's own convention, and unknown scene fields stay ignored because the scene spec at version 0.1 keeps forward compatibility with fields it does not yet define. One audit observation was likewise recorded rather than fixed. The investigation's latency finding can misattribute its subject under duplicate span ids, naming one copy's service against another copy's self time, on malformed input only and with correct totals.

The surfaces the audit cleared are part of the record. The single SVG serializer escapes every caller value at emission, the HTML export and the MCP view interpolate no caller text, the scene validator names its refusals field by field, the JSON-RPC handler's error discipline held on every malformed shape, and the sankey staging, service layering, percentile, and RED arithmetic re-verified clean under the combined duplicate, cyclic, and absent-parent inputs this round was chartered to try. The standing lanes discharged as the design directs. The A2UI watch was reviewed and stays a watch item, and the convention check rests on the same day's charges pass.

## Round 16: one id, two spans (2026-11-23)

The round closed the one observation Round 15 recorded without a fix, under the charter in docs/DUPLICATE-ID-DESIGN.md. Two spans in one trace can carry the same span id, a shape real pipelines produce on purpose when a client span and its server span share an id in the Zipkin model and a bridge carries both into OTLP. The recon for the design found the damage wider than the finding that named it. The waterfall builder deduplicated by id and dropped the second copy entirely, and the three surfaces that join rows back to spans each picked a different copy, so one trace could carry three answers to the question of which span a row describes. Attribution is now by occurrence, and INV-89 states the law.

WaterfallRow carries the position of the span occurrence it was built from in the slice the caller passed, beside the id it already carried. The waterfall and flame walks track occurrences rather than ids, so every copy emits its own row and builds its own node with its own figures. Children attach to the first occurrence of their parent id in canonical order, because the copies cannot be told apart as parents, and the later copies carry the self time of their own intervals alone. The SVG waterfall, the MCP panel detail, and the investigation's latency finding all join rows to spans by the occurrence index, and the latency finding's total now counts every occurrence's self time. The finding's critical path clause stays id-based, because a critical path is a chain of ids by definition and its stop rule already defines what happens when an id repeats.

The audit the design fenced in covered the remaining walks. The flame builder crossed in the same way as the waterfall and took the same occurrence fix. The service graph resolved a shared parent id to whichever copy arrived last, so a child's caller could be the wrong copy's service, and the parent now resolves to the first occurrence in canonical order like everywhere else. The sankey staging already chooses one canonical copy per id by a stated rule that compares the spans' own fields, so its choice never depends on arrival order and never crosses figures between copies, and it stands as defined behavior. The RED aggregation groups each span's own duration under its own service and joins nothing by id, so it stands as well.

One discovery sat outside the design's text and the build reconciled it rather than routing around it. The MCP input assembly and the CLI's combined set both deduplicated spans by trace id and span id alone, under the one-count-per-span and counts-once rules of INV-85 and INV-86, which would have dropped the second copy before any surface could render it. Those rules govern spans that arrive twice, and a byte-identical re-delivery is one span delivered twice, while two different spans under one id are two spans. Both boundaries now run one shared whole-span dedup from the scene crate, so a re-delivery still counts once and distinct copies both survive, and the incident pin that guards the re-delivery rule still passes.

The red team then found the one shape where the design's first two clauses conflict, and the round amended the design rather than shipping the conflict. When the first copy of a duplicated id sits inside a parent cycle and a later copy hangs from a root, hanging the children from the first copy loses them, because no walk reaches it. Children now attach to the first copy in canonical order, or to the first reached copy when that copy is unreachable when adoption settles, and the row promise covers every reached occurrence. Adoption settles stickily as the reachability closure promotes, so in a circular shape a first copy can end up reached through another copy's adoption yet never adopt itself, which is the only consistent reading of the shape and costs no span its row. A copy trapped in a parent cycle with no reached copy of its id keeps the standing cycle fate and builds nothing. A pin builds the shape and asserts every span in the trace earns a row, with the cycle breaking at the adoption seam, and the design record carries the amendment beside the clause it qualifies. Two smaller review notes stand recorded rather than repaired. The latency finding's tiebreak orders by span index after span id, which makes its winner a total order and can name the other trace's copy on an exact cross-trace tie where the old comparison kept the first it met. The flame root sum saturates where it once added plainly, the direction INV-88 already mandates for extreme figures.

Two joins of the same class survive on surfaces this round fenced out, and they are named here so the next recon does not have to rediscover them. The dialect session walk in the ingest crate keys its children map and its emitted set by span id, so a second copy of an id emits no step and two root copies of one id can each walk the same merged children into separate turns. The Claude Code fusion wait joins blocked children to a tool step by the tool span's id, so a wait belonging to one copy's children can land on a step contained in the other copy's window. Both are the F7 class on the agent-run surface, and they are the next round's opening recon.

Three standing tests restated their assertions on the new rule, at full strength. The two cycle tests now assert the occurrence shape of the same cyclic fixture, a four-row walk and a four-node tree where the repeated span builds as a leaf, with termination carried by the shape as before. The input-order test now compares the row shape apart from the join index and asserts the index resolves to its own span in each input. The hosted gate holds 212 tests, five of them new pins, and the golden corpus and scale hashes stand unchanged, because every committed fixture carries unique ids within its traces. Adding a field to the public WaterfallRow struct is a shape change in a 0.x crate whose only constructor is this workspace, and the next release carries the version that says so.
## Round 17: the session derivations under INV-89 (2026-11-23)

The round closed the two joins Round 16 recorded as named leftovers, under the charter in docs/SESSION-ID-DESIGN.md. Both recon lanes verified the reports firsthand before any code moved, and the session side proved worse than the trace side, because the session model prices what it walks. In the dialect walk, the children of two copies of one span id merged into one list either copy could be handed, the visited set keyed on the id suppressed the second copy's step entirely, and a suppressed model copy took its tokens and cost with it, uncounted anywhere. Two root copies of one id each built a turn over the merged children, so the shared subtree landed in both turns and the session headline double-counted it. In the Claude Code fusion, the blocked map pooled every blocked_on_user child under its parent span id across copies, and a step took the union sum of both copies' blocked children as its permission wait, on disjoint windows, because the gate was per occurrence while the pool was per id.

No new invariant joins the set. INV-89 already stated the law for this class, and the build brought the session derivations under it, extending its enforcement line to the ingest suites. The dialect walk now works on span positions in the caller's slice throughout. Its visited set keys on the occurrence and spans the whole trace rather than resetting per turn, so every copy emits its own step exactly once and no subtree walks twice, and termination on parent cycles rests on the same finiteness it always did. The children of a repeated id are consumed by its adopting copy alone, resolved through a shared helper in the scene crate that exposes the trace walks' adopting computation, so the rule is one rule in one place. Turn coverage counts reached occurrences, which makes it agree with the steps the turn actually carries. The fusion resolves each blocked child to the tool span occurrence whose window contains it, the smallest such window when copies overlap, and keys the blocked map by that occurrence, so a step's wait is its own occurrence's sum and a child no occurrence contains stays unattributed. The merged coverage counts in both fusions count occurrences rather than id strings.

Two observations from the recon stand recorded with their dispositions. The prompt id and agent id labels remain the source dialects' own strings, so two turns can share a prompt id label, because labels are display and never join keys. The decision maps in both session adapters key a tool's decision by turn and tool name, so two decisions for one tool name in one turn collapse to the last. That is a name keyed collapse, a different class from an id join, and no evidence in this round shows a real export carrying two decisions for one tool in one turn, so it stands recorded rather than built on a guess. The Codex adapter carries no span id join and did not change.

Three pins carry the round, and one standing test restated its assertions on the new rule. A GenAI session with two copies of one id and children naming the shared id emits both copies as subagent steps with the children under the adopting copy and the totals counting each call once. Two root copies build two turns that do not double the shared subtree. A Claude Code session with two tool spans under one id on disjoint windows gives each step its own occurrence's wait. The cyclic session test now asserts the occurrence shape of its fixture, four occurrences counted and the repeated span emitting its own childless subagent step, with termination carried by the shape as before. The hosted gate holds 217 tests, and the session fixtures stand unchanged, because every committed fixture carries unique ids within its traces.

One deviation from the design record stands accepted, named in its amendment section. The design fenced the round with byte-identical output on unique-id input, and the red team found one narrow corner where the build differs. A blocked child whose window falls outside the tool span its parent id names was pooled under the id and credited to steps in that span under the old rule, and under the window-resolved rule it stays unattributed, so the wait and the merged count change on such input. The old figures credited a permission wait to an invocation that provably does not contain the blocked interval, and on well-formed unique-id input the fence holds exactly. The overlapping-window tie-break in the fusion, smallest containing window first, is defined in the code comment and stands unpinned, the disjoint shape being the one the design named.
## Round 18: decision pairing and the fusion tie-break (2026-11-23)

The round closed the two observations Round 17 recorded, under the charter in docs/DECISION-PAIRING-DESIGN.md, and both recon lanes refined their subjects before any code moved. The decision maps in the Claude Code and Codex adapters keyed a tool's permission decision by turn and tool name, with a later decision overwriting an earlier one. The recon's precision was that a step reads the map at its own point in the single event pass, so a strictly sequential turn was already correct and the crossing needs the batched interleaving concurrent tool calls produce, both decisions logged before either result lands. No invocation identifier exists in either dialect's events to join on. The signals are turn, tool name, and event order, and nothing else.

Decisions now pair to results by occurrence within a turn and tool name, under the new INV-90. Each adapter's map holds a queue per key. A decision event appends, a result takes the oldest unconsumed decision for its name, a result with an empty queue carries no decision, and a decision no result consumes labels nothing. The change is one type and two call sites per adapter and rides the queue idiom the Codex adapter already used for its time-to-first-token join. The residue is the one the design stated. Pairing follows emission order because emission order is the only order the export carries, so an export that logs results in a different completion order than its decisions can still mismatch a label. The rule strictly dominates the old one, which mislabeled even when the counts matched and only the interleaving was batched. One pin per adapter builds the batched interleaving and asserts the results read the decisions in order, and the whole committed corpus stays byte-identical because every fixture carries at most one decision per turn and tool name or interleaves strictly.

One fence qualification stands recorded in the design record's amendment, found by the red team. The design's neutrality sentence covered any stream with at most one decision per turn and tool name, and one sub-shape inside that wording differs. With one decision followed by two results for the same name, the old map's read did not consume, so both results carried the decision, while the queue labels the first and the second carries none. That is INV-90's defined behavior rather than a defect, because a single permission decision labeling two distinct invocations is the crossing class in milder form. The fence holds for count-matched streams, for strictly interleaved streams, and for the whole committed corpus.

The fusion's smallest-containing-window tie-break needed no code, only its pin. The rule was already precise and deterministic, resolving a blocked child among the containing occurrences of its parent id to the smallest duration and then the earliest start, with exact ties falling to the earliest occurrence in the input. The new pin builds the nested shape the recon designed, two tool spans under one id with the larger first in the export and one blocked child inside both, and asserts the step only the larger occurrence contains keeps a zero wait while the step inside both takes the child's duration. Those two assertions together kill the largest-window, earliest-start, first-in-input-order, and pooled-by-id rules, and the nonzero wait kills dropping the child as ambiguous.

Three adjacent choices in the same fusion block stand recorded as defined behavior in the design record, unchanged by this round. A step contained in two tool occurrences takes the larger of the two per-occurrence sums as its wait. Children enter merged coverage only when their occurrence's sum actually sets a step's wait. The wait fusion contains inclusively at both ends while the subagent fusion captures by midpoint with a half-open end. The hosted gate holds 220 tests.
## Round 19: the 0.4.0 release (2026-11-23)

All five crates are published at 0.4.0 on crates.io, each confirmed live on the index before the next was sent, and all five builds stand green on docs.rs. The release carries Rounds 16 through 18. Round 16 added a public field to WaterfallRow, a shape change in a 0.x crate, so the release is a minor bump rather than a patch, and the version moved together across the five manifests and the three workspace dependency pins in PR #56 before any crate was packaged.

The packaging repeated the proven route. A scratch dispatch branch carried the vendored-staging workflow under its registered name, a hosted run packaged the five crates at the versions their manifests declare, and the packaged files were published one at a time in the settled order, scene, ingest, svg, telorama, telorama-mcp. The hosted gate holds 220 tests on the released tree.
## Round 20: the verdict viewer (2026-11-23)

The verdict-viewer treatment David ruled for the fleet's showcase surfaces, crosscheck first and telorama next, now stands for telorama as its own surface under docs/VIEWER-DESIGN.md. A web directory beside the crates carries a Next.js static-export page in the crosscheck viewer's stack and shape. It renders one recorded run and computes nothing at view time. The incident scene stands large with its investigation findings beside it, and a recorded Claude Code session follows as a frame stream whose stepping replaces whole frames. A summary bar carries the finding counts per kind and the investigation's own headline. An inspector shows the selected finding's statement, its evidence, its exemplars, and the panel it is shown in. The footer prints the limit line, that the page proves the record matches the tool output byte for byte and nothing more.

The record under web/data is assembled by new tests in the MCP suite from the committed fixtures through the same library paths the tools use, and the freshness test asserts the committed record equals the current output byte for byte, the gallery pattern carried to the new surface under the new INV-91. Two phrases in the design met the code during the build and their settlements stand in the design record's amendment. The investigation layer has no severity, so the classes are its four finding kinds ordered by rank. Findings carry no panel field and the delivered SVG carries no anchors, so the generator derives each finding's panel from its kind and the scene's panel list and the record carries the anchors as data, the page marking by panel reference rather than inside the SVG. Under that derivation the incident heatmap is anchored by no finding, the quiet panel the corpus requires, and the gate asserts it. The corpus covers every kind between its two halves, errors and latency from the incident, wait and cost from the session frames.

The gallery, the standalone HTML export, and the MCP Apps view are untouched, and no crate's rendering, derivation, or tool surface changed. The page injects the delivered SVG as markup under the same trust the MCP Apps view already places in the renderer's output, with the freshness gate standing between the record and any hand edit. The hosted gate holds 222 tests. Hosting the static export stays out of scope, as it does for crosscheck, and a money-shot video of the viewer belongs to a later showcase pass under the Stageshow standard.
## Round 21: anchor truth hardening (2026-11-24)

The round closed the exposure Round 20's red team recorded in the viewer design record. The record generator's anchor derivation no longer reads a finding's kind alone. The latency anchor now requires the waterfall panel's data entry to carry the finding's exemplar trace, and the errors anchor requires the error rate row to be unfiltered or filtered to the finding's service. A panel that draws something else earns the finding no anchor rather than a false one. A synthetic test mutates the composed incident scene three ways, a different exemplar trace, no trace at all, and a row filtered to another service and then to the subject, and pins the withholding and the positive cases. The frame findings pins in the coverage test moved from containment to exact vectors, since a frame investigates its run alone and can emit exactly the session kinds.

On the committed corpus the derived anchors are unchanged, so the record bytes stand and the freshness gate holds on the same record. The hosted gate holds 223 tests.
## Round 22: the viewer scene reel (2026-11-24)

The showcase pass for the verdict viewer took the honest artifact the box can produce today. No shell-drivable browser exists here, and the hosted Chromium is policy-gated at startup, so a capture of the page itself, with its clicking and inspector, is still owed and is recorded as owed rather than simulated. What shipped is a scene reel at docs/media/viewer-reel.mp4, assembled with ffmpeg from the viewer record's own renderings and nothing else. The incident scene holds in two halves at the panel boundary, then the recorded session's two frames replace each other whole, which is the only motion the product itself makes. The reel is captioned as a scene reel wherever it is referenced. It is not a capture of the viewer page and does not pretend to show the findings list, the inspector, or the summary bar.

The reel's frames are the record's committed SVGs rasterized at the same 1160-wide scale as the existing showcase media, so what the reel shows is byte-derived from the freshness-gated record Round 20 landed. The page capture joins the backlog beside the MCP Events watch, to be taken when a drivable browser is available to this seat.

## Round 23: the Codex session in the viewer record (2026-11-24)

The verdict viewer's recorded run now carries both priced session dialects the engine ingests. The record's session field became a sessions list holding the Claude Code session and the Codex session from its priced fixture, assembled through the Codex adapter with an empty spans slice, the CLI frames shape for a logs-only dialect. The page renders one section per session, names each dialect from the record, steps each session's frames independently, and counts the incident plus each session's final frame in the summary bar under a scope label that says so. INV-91 is amended in place to name the sessions as part of the recorded run.

The build took the measurement the design record reserved. A Codex frame carries exactly one finding kind, cost, because the Codex adapter merges no wait data, and the coverage test pins exactly that instead of borrowing the Claude Code shape. The incident half and the Claude Code frames regenerated byte-identical. The hosted gate holds 223 tests.

## Round 24: all four dialects in the viewer record (2026-11-24)

The verdict viewer's recorded run is complete. The GenAI and OpenInference sessions joined the record's sessions list in CLI order, each assembled from its base spans fixture through its spans-only adapter in the same mold as the first two entries. The page needed no code change, because its sections, steppers, and summary loop already generalize over the list, and INV-91 needed no amendment, because Round 23 already worded it as one recorded session per dialect the record carries.

Each new session carries one frame whose only finding kind is cost, a GenAI turn at $0.0231 and an OpenInference turn at $0.0094, both on gpt-5.1, because neither span dialect merges wait data. The coverage test pins exactly that with the reason stated. The summary bar's totals on the completed record are one errors, one latency, one wait, and four cost. The incident half and the two existing sessions regenerated byte-identical. The hosted gate holds 223 tests.

Two items stay owed beside the MCP Events watch. A capture of the viewer page itself waits on a drivable browser being available to this seat, and the local export verification of the enlarged page waits on a calm window in the box write fault. Both are recorded as owed rather than simulated or assumed.

## Round 25: the scene reel for the completed record (2026-11-24)

The scene reel at docs/media/viewer-reel.mp4 was refreshed for the completed record. The Round 22 reel showed the two sessions the record carried then, so it understated the viewer after Rounds 23 and 24. The new reel is assembled with ffmpeg from the completed record's own committed SVGs and nothing else, in the Round 22 composition. The incident scene holds in two halves at the panel boundary, then all six session frames replace each other whole in record order, the two Claude Code frames, the two Codex frames, and the single GenAI and OpenInference frames. It runs 30 seconds at 1160 by 868.

The reel remains a scene reel, not a capture of the viewer page. It does not show the findings list, the inspector, or the summary bar, and the page capture is still owed beside the MCP Events watch, to be taken when a drivable browser is available to this seat. The README caption needed no change, because it already describes the reel as showing the record's incident scene and session frames exactly as the tools rendered them.

## Round 26: the MCP Events watch re-check (2026-11-24)

The MCP Events watch was re-checked firsthand and its disposition is recorded in docs/EVENTS-WATCH.md. SEP-3415 stands at Final in the Extensions Track, merged on 2026-11-19, with the extension published at Beta for protocol revision 2026-07-28 and later. The watch therefore changes basis. It is no longer a draft that might never land, and the open question is whether telorama ever has anything for it to carry.

It does not today. telorama-mcp fronts no upstream system, nothing happens inside telorama between calls, and no source exists that an events/list response could honestly name. The watch continues with two named triggers that would reopen it as a design question, telorama gaining an upstream source of its own, or the extension leaving Beta with SDK support paired with a designed watcher mode. The re-check cadence is the next MCP protocol revision or the extension leaving Beta. No code changed in this round.

## Round 27: the README accuracy pass (2026-11-24)

Every checkable claim in the README was audited against the repository, forty-seven claims in all, each verified against the code, the tests, the manifests, or the crates.io record. Forty-five held exactly as written. Two were stale, both in the closing summary, and both are corrected in this round. The roadmap count said thirteen rounds and now says twenty-six, the count of the rounds recorded in this file before this pass. The published version said 0.3.0 and now says 0.4.0, matching the crate manifests and the crates.io record for all five crates.

Nothing else in the README changed. The corpus counts, the scale proofs, the dialect list, the panel list, and the nesting budget all verified as written, so the pass corrected only what the audit proved stale and left the rest of the file untouched.

## Round 28: the docs surface accuracy sweep (2026-11-24)

The accuracy treatment of Round 27 was extended to every remaining public document, the four example walkthroughs and their index, the plugin README and skill, the web README, the DOSSIER, the framing of the invariants file, and the five crate manifest descriptions. Three claims proved stale and are corrected here, each with the smallest change that makes it true.

The agent session walkthrough said scenes named for each dialect sit in the scenes fixture directory. Three do, for Claude Code, Codex, and GenAI, and no scene named for OpenInference exists there, so the sentence now names the three. The DOSSIER pointed readers at six research files in its own directory, and none of the six is in the repository, so the pointer now says the fronts are kept outside the repository and names their subjects instead of their files. The invariants framing said its sections run in invariant number order, and they do not, because the closing hostile-input section carries INV-82 through INV-84 after the documented surface section ends at INV-91, so the sentence now describes the grouping as it stands.

Everything else verified as written, including every command and fixture path in the walkthroughs, the plugin's count of seven tools, and the invariants file's one to one tracing of its ninety-one entries to their enforcements. No code changed in this round.

## Round 29: the OpenInference golden scene, design (2026-11-24)

A design record at docs/OPENINFERENCE-GOLDEN-DESIGN.md covers the one asymmetry the Round 28 sweep exposed. The golden corpus carries an agent run scene for Claude Code, for Codex in two variants, and for GenAI, but OpenInference has no scene fixture and no golden of its own, only adapter coverage and its place in the viewer's recorded run. The design adds the smallest artifact that closes the gap, a scene fixture mirroring the GenAI one, a golden test cloned from the GenAI test, and a golden generated on the hosted route through the UPDATE_GOLDENS mechanism, with the figures it must carry already measured from the viewer record's rendering of the same session.

The fences are stated in the record. No production code changes and no new invariant, and the build moves the README's corpus count from thirteen scenes to fourteen in the same change. The build round follows on this design.

Review amended the design in one place before it landed. The record first fenced the new scene out of the gallery, and the red team showed that fence cannot hold, because the gallery coverage gate at INV-63 asserts every scene fixture has a gallery section. The design now brings the scene into the gallery instead, with a section added to the gallery test's list and the gallery page regenerated on the hosted route in the build. The same review pass corrected the golden discipline citation to INV-18 and stated the scene range pad numerically.

## Round 30: the OpenInference golden scene, build (2026-11-24)

The build landed on the Round 29 design as amended. The corpus now carries an agent run scene for all four dialects. The new scene fixture mirrors the GenAI one with the OpenInference facts in place, one agent_run panel over an openinference_session source, its range opening at the fixture window's first span and closing 3000 milliseconds past its last. The golden test cloned from the GenAI test adapts the session fixture with parse_openinference_session and pins the render, and the golden itself was generated on the hosted route through the UPDATE_GOLDENS mechanism, never by hand.

The generated golden carries every figure the design measured from the viewer record's rendering of the same session. One turn of 2900 input tokens, 570 output tokens, and 600 cache read tokens, priced at 0.0094 dollars on gpt-5.1 as telorama's estimate, the calculator tool call in the run, and the coverage line counting no log events, five spans read, and five spans merged. The scene also joined the gallery, as the amended design requires, with a section added to the gallery test's list and the page regenerated on the hosted route through the UPDATE_GALLERY mechanism. The README's corpus count moved from thirteen scenes to fourteen in the same change, the corpus now standing at fourteen light scenes with the five dark pins unchanged, and the suite stands at 224 tests. No production code changed and no invariant was added.

## Round 31: honest text width, design (2026-11-24)

A design record at docs/TEXT-WIDTH-DESIGN.md takes up the one item the records still name as open and unbuilt. The DOSSIER's open risks have carried it since the research programme. Latin-subset fonts fail CJK and Arabic service names, and the mitigation was left as an open spike. The design locates the failure in the renderer's own layout law, which budgets every label by character count. A CJK character takes two columns and counts once, so a Chinese or Japanese service name overflows the column reserved for it, and a combining mark takes no width and counts once, so accented names can be cut mid sequence.

The design answers with one pure function that reads a string's layout width from its characters' classes, zero for combining marks, two for the East Asian wide and fullwidth ranges, one for everything else, with truncation and budgeting converted to width at every site a census finds. Its proving fence is that Latin layout cannot move, because the function returns exactly today's count for the text the corpus already pins, so every existing golden must pass byte for byte. Review added two precisions to the record before it landed. The ambiguous class is named as counting one, since the corpus's own separator and ellipsis are ambiguous characters, and the truncation conversion keeps today's edge behaviour at budgets of zero and one. The record also states what the fix does not claim. Shaping and bidirectional ordering belong to the text engine that renders the SVG, not to this layout model. The entry closes with the audit that chose the arc. The dated snapshot normalization closed under INV-84, the api_refusal convention under INV-83, and the findings divergence under INV-81, while the A2UI emitter and MCP Events stay behind fences whose triggers have not fired. The build round follows on this design.

## Round 32: honest text width, build (2026-11-24)

The build landed on the Round 31 design. The renderer now measures every label by character class under INV-92. A width function in telorama-svg counts combining marks and the zero width characters as zero, the East Asian wide and fullwidth ranges as two, and everything else, the ambiguous class included, as one, over hand-derived range tables whose sources the code states. The truncation helper cuts to a width budget at a character boundary and never splits a combining sequence, and the one layout site that positioned text by a raw count now positions it by width. The census the design required found eleven sites in the one file, five truncation call sites, the pixel derivations that feed their budgets, and that single positioning site, and every one of them now flows through the width model.

The proving fence held. Every golden the corpus already pinned passes byte for byte, because the width of the Latin and ambiguous text they carry is the count it always was. The new proof is a trace named in four scripts, CJK, Arabic, a Latin name carrying a combining mark, and a Latin control, with a golden scene over it whose longest service name runs to twenty wide characters and now truncates inside its column. The scene joined the gallery and the corpus count moved from fourteen scenes to fifteen, the corpus standing at fifteen light scenes with the five dark pins unchanged. Unit tests in the renderer pin the width classes at their range edges and the truncation edges the review named. The suite stands at 230 tests, and no public API changed.

## Round 33: completing the width tables, design (2026-11-24)

Round 32's red team replayed the landed width tables against the Unicode data across every codepoint and counted 458 disagreements, none touching a character any label in this repository carries, concentrated in non-spacing marks of recent and minority scripts the hand table omits and in wide codepoints it counts as one. This round designs the completion, and its record stands at docs/WIDTH-TABLE-COMPLETION-DESIGN.md.

The completion replaces the hand-derived principal blocks with tables derived from the Unicode Character Database through a named authority whose Unicode version the build record states. The width rule itself does not change, so INV-92 stands unamended. Three classes of remaining disagreement are intended and will be enumerated in the build record rather than rediscovered later, the unassigned codepoints no name can contain, the marks that are both non-spacing and wide counting zero as the wcwidth tradition has them, and whatever a future Unicode version adds until the derivation is re-run. The proving fence is Round 32's own, every pinned width identical and every golden byte for byte, joined by the sweep run as the build's verification, done only when the disagreements left are exactly the principled remainder. The design also answers the pin precision note Round 32's review left standing, keeping the hand-checkable unit pins and resting the exhaustive claim on the replayable sweep recorded with its counts. Review amended the design in four places before it landed. The named zero width format list, U+200B, U+200C, U+200D, and U+FEFF, is now enumerated in the design and held to land verbatim, since category data alone cannot derive it. The sweep's founding count is stated with its precedence, 458 against the wide-first naive reading and 453 under the rule's own zero-first precedence, and the residue beyond the named classes is accounted as hand-drawn range edge artifacts. The pin substitution is named as an amendment to the Round 31 design's pin discipline, with its cost stated. And the scope section now states the rendering boundary, that a handful of default-ignorable format characters count one under the rule while browsers advance them zero, and that counting them zero would be a change to INV-92 itself rather than a table completion. The build round follows on this design.

## Round 34: completing the width tables, build (2026-11-24)

The build landed on the Round 33 design. The two width tables in telorama-svg are no longer hand-derived principal blocks. They are derived from the Unicode Character Database as published by the unicodedata module of Python 3.12, Unicode 15.0.0, the zero width table taking every codepoint whose general category is Mn or Me together with the named format characters, and the wide table taking every codepoint whose East Asian Width property is W or F. The zero width table now carries 345 ranges against the hand table's 294, and the wide table 121 against 73. The named format list landed verbatim, U+200B to U+200D and U+FEFF, and seven codepoints sit in both tables with zero winning by the rule's precedence, the six non-spacing wide marks the design named and U+16FE4 beside them. Unassigned codepoints in the reserved ranges carry the width the data gives them, which is the choice the design required this record to state.

The proving method ran as designed. The sweep replayed the landed tables against the derivation source across every codepoint under the rule's zero-first precedence and found no disagreements, where the hand tables under the same reading disagreed 453 times. The byte fence held with it, every golden, the gallery page, and the viewer record unchanged, because no width the corpus pins has moved. The derivation ran on the hosted route, and its version check earned its place on the first attempt, when the runner's own Python carried Unicode 16.0.0 and the generation refused rather than derive from the wrong source, after which the workflow pinned Python 3.12 and the derivation ran clean. As the Round 33 design amended the Round 31 pin discipline to provide, the unit pins stand as they were and the recorded sweep is the exhaustive guard. The suite stands at 230 tests, and no public API changed.

## Round 35: version 0.5.0, release (2026-11-24)

The five crates move from version 0.4.0 to version 0.5.0, and the release carries the honest text width work of Rounds 31 to 34. A service named in CJK, Arabic, or any script whose characters run wider or narrower than one column to a character now lays out at its true width, with truncation that never splits a combining sequence, over tables derived from the Unicode Character Database rather than drawn by hand. Nothing else in the public API has changed since 0.4.0, and the corpus the release renders is the fifteen-scene corpus with its five dark pins. The README's round count moves from twenty-six rounds to thirty-four and its version line to 0.5.0 in the same change. The crates publish in the established order, scene, ingest, svg, telorama, and telorama-mcp, through the banked packaging route, and the publish is verified against the crates.io index before the round is reported closed. The viewer record's engine version stamp moves to 0.5.0 with the release, the record regenerated on the hosted route with its renders and hashes unchanged.

## Round 36: the scale re-proof, design (2026-11-24)

The design record at docs/PERF-REPROOF-DESIGN.md states the re-proof of the Round 14 scale figures against current main, now that Rounds 32 and 34 changed the path every label takes through the renderer. The three workloads are the frame stream at five thousand turns, the ten thousand span GenAI parse, and the ten thousand event Claude Code parse, all three already running under the normal CI job, and the round harvests repeated timings for them on the hosted route under a stated method of medians and spreads over pooled samples. Its fences are stated in advance. The final frame must print at 4,150,019 bytes under its pinned hash on every sample or the round stops on a width leak finding. The parse medians record as unchanged at or under twice their Round 14 figures and count as regressions only over tight pools. The frame build median records at or under one and a half times its Round 14 figure and counts as a defect only over a tight pool, with wide pools at five dispatches landing an unanswered statement rather than a number. The INV-70 ceilings are never edited. The design changes no production code and no test code, does no optimisation work, and lands nothing but its measurement entry when the fences hold.

## Round 37: the scale re-proof, measurement (2026-11-24)

The re-proof ran under the method the Round 36 design states, three dispatches of the measurement workflow on ubuntu-latest, pooling fifteen frame samples and twenty-one samples for each parse, every sample its own invocation under the exact test name. Byte identity held on every frame sample. Each one printed the final frame at 4,150,019 bytes under the hash c5ed954ad4390b9f, with the mid-stream hash asserted by the test itself on every run, so the width work of Rounds 32 and 34 did not move Latin rendering at scale. The frame build median stands at 8.13 seconds with a spread of 7.54 to 11.46 seconds, against the Round 14 figure of 13.06 seconds, inside the fence of one and a half times the old figure and recorded as the new figure beside the old with no mechanism claimed for the difference. The GenAI parse median stands at 83.1 milliseconds with a spread of 78.4 to 122.3 milliseconds, against the Round 14 figure of 47 milliseconds, inside the fence of twice the old figure and recorded as unchanged within runner noise under the rule the design stated in advance. The Claude Code parse median stands at 157.4 milliseconds with a spread of 114.3 to 196.6 milliseconds, against the Round 14 figure of 176 milliseconds, likewise inside its fence and unchanged within runner noise. Every pool was tight under the design's test, the INV-70 ceilings stand untouched, and the measurement branch and its workflow never reached main.

## Round 38: the wide names dark pin (2026-11-24)

The wide names scene gains a dark golden pin beside its five dark companions. The pin renders the same CJK, Arabic, and combining-mark names under the dark theme, so the width behaviour of Rounds 32 and 34 is pinned in both themes rather than under the light theme alone. The golden is generated on the hosted route. Nothing else changes. No production code moves, no new invariant is added, and the corpus stands at fifteen light scenes and six dark pins.

## Round 39: the README accuracy pass (2026-11-24)

The README was audited claim by claim against the tree on main and the crates.io index, in the pattern of the Round 27 pass. Fifty-five checkable claims were audited and fifty-four stand true, among them the corpus at fifteen scenes with six of them pinned in both themes, the nine panel kinds against the Panel enum, the seven MCP tools named one for one, the four dialects, the nesting budget of 512, the scale proof at ten thousand spans and a hundred thousand samples, and all five crates published at version 0.5.0 with none yanked. One claim was stale. The round count read thirty-four rounds with thirty-eight complete, stale by exactly the four rounds that landed since the release set it, and it now reads thirty-eight, the count of rounds completed before this one under the standing convention. Nothing else in the README needed to change, and the audit found no invariant count or docs.rs claim in the README to go stale.

## Round 40: the docs surface accuracy sweep (2026-11-24)

The documentation surface beyond the root README was audited claim by claim against the tree on main, repeating the Round 28 sweep after the rounds that changed the corpus, the version, and the width story. Thirty-six checkable claims about the present state of the repository were audited across the per-crate READMEs, the web and plugin documents, the examples, the DOSSIER, the watch documents, and the INVARIANTS framing, and thirty-four stand true. Two claims were stale and both are corrected in this round. The agent session example named scenes for three dialects, a sentence Round 28 had narrowed when no OpenInference scene existed, and it now names all four dialects since the OpenInference scene landed in Round 30. The DOSSIER carried the CJK and Arabic name risk as an open spike, and it now records that Rounds 31 to 34 closed the width half of that risk under INV-92, with text shaping and bidirectional ordering still outside the renderer by the Round 31 fence. The twenty-six design records under docs were read as the round history they are and none makes a false claim about the present tree. The sweep also observed that the roadmap document's coverage ends at Round 15, which asserts nothing false about the present state and is left as it stands.

## Round 41: the roadmap brought current (2026-11-24)

The roadmap document's coverage ended at Round 15, twenty-five rounds short of the present, and the Round 40 sweep recorded the gap as an observation rather than a stale claim. This round closes it in the smallest way that makes the document honest. The roadmap gains one closing section, set before its standing lanes, that records the programme's arcs from Round 16 to Round 40 and names DECISIONS.md as the live account of the programme. No existing roadmap section was rewritten and no plan prose was written for rounds already done. Every arc the new section names was derived from the DECISIONS entries for those rounds, read firsthand for this round, and the two releases it names are the 0.4.0 release of Round 19 and the 0.5.0 release of Round 35.

## Round 42: dialect and price currency re-checked (2026-11-24)

The price table and the GenAI dialect were re-derived against their current published sources, repeating the Round 3 currency pass. Every verifiable figure in the price table stands current across its twenty-six priced model ids. The Anthropic rows match the platform documentation pricing page and the OpenAI rows match the per-model pages, the main pricing page no longer listing the GPT-5 family at all. Several of those models now carry deprecated or shut down status on their pages, and their rows stay, because the figures remain the last published prices and historical traces naming those models still price correctly from them. One row, gpt-5.1-mini, has no current published price anywhere, and it stays on the Round 3 rationale that traces name it and the gpt-5-mini row corroborates its figures. The table's as-of date stays at 2026-11-23, the date its figures were compiled, and this entry carries the re-verification. The GenAI dialect stands current against its pinned conventions. The seven commits since the pin touch nothing the adapter reads, every attribute it maps stands in the current registry under its exact name with nothing renamed, removed, deprecated, or newly required, and the pin's premise holds, the conventions repository having still cut no versioned release. Newly published models carry no rows, the GPT-6.2 and GPT-5.4 families and the newer Fable and Mythos lines among them, and adding rows waits on fixtures under the standing rule that a shape is fixtured before it is supported. The Claude Code api_refusal event remains the named candidate for the next convention pass, its fields still unverified from a live export. No code changes in this round.

## Round 43: price rows for the current model lines, designed (2026-11-25)

Round 42 found the table current for every model it prices and silent on the models published since. This round designs the rows for the current lines, and docs/PRICE-ROWS-DESIGN.md carries the design. Eleven model ids gain rows in eight arms, every figure re-fetched from the vendor page the design names, the GPT-6.2 pair, the GPT-6 and GPT-6.1 Sol and Astra ids, the GPT-5.4 family in its three priced sizes, and the Fable and Mythos lines at 5 and 5.1. Three exclusions are settled with their reasons. The pro size prints no cached input rate and lands no row, the Terra and Luna rates stand unverified on any vendor page and wait for the next currency pass, and Haiku 5.5 keeps its tiered rate outside the flat row shape. The Codex rows carry a gate. No codex-suffixed GPT-6.2 id exists on any vendor page, so the build round keys the rows to the model string a real Codex export of a GPT-6.2 run actually carries, and if no real export can be staged those two rows wait while the rest of the round lands. The design states the as-of rule the record has been acting on, that the date names the compilation of the table's contents and advances when the contents change and not on a re-verification alone, and it fences the build so no existing figure moves, the four goldens that print the date moving in that line only. A new invariant, INV-93, is drafted to state the admission discipline INV-68 does not, that a model id carries a row only when a committed fixture names it and a vendor-published price stands behind every figure. The build follows as Round 44.

## Round 44: price rows for the current model lines, built (2026-11-25)

The Round 43 design lands for nine of its eleven ids. Seven arms join the price table in crates/telorama-ingest/src/pricing.rs, covering gpt-5.4, gpt-5.4-mini, and gpt-5.4-nano, gpt-6-astra and gpt-6.1-sol, and the Fable and Mythos lines at 5 and 5.1, each figure the one its vendor page published and the design named. Two fixtures prove the rows, a GenAI session naming the five OpenAI ids and an OpenInference session naming the four Claude ids, each with its scene, its golden, and its gallery section, and each run pricing in full with nothing left unpriced. The table date advances to 2026-11-25 under the Round 43 rule, because the contents changed, and every surface that prints the date moves with it while no existing figure moves. INV-93 now stands in the Dialect currency section, stating that a model id carries a row only when a committed fixture names it in a dialect export and a vendor-published price stands behind every figure. The GPT-6.2 pair does not land. The design gates those rows on the model string a real Codex export of a GPT-6.2 run carries, no such export could be staged for this build, and the pair's ids are pinned rowless by the pricing tests until one can. gpt-5.4-pro stays rowless beside them, its page printing no cached input rate.

## Round 45: the README accuracy pass, third running (2026-11-25)

The README was audited claim by claim against the tree for the third time, in the pattern of Rounds 27 and 39. Thirty-five checkable claims were audited and thirty-three stand true, among them the nine panel kinds, the seven MCP tools, the four dialects, the six dark pins, the nesting budget of 512, the scale proof, and all five crates at version 0.5.0. Two claims were stale, both in the current state paragraph and both staled by the rounds that landed after the Round 39 pass. The round count read thirty-eight with forty-four complete, and it now reads forty-four, the count of rounds completed before this one under the standing convention. The golden corpus read fifteen scenes, and it now reads seventeen, the corpus counted in light goldens as every earlier pass counted it, with Round 44's two new scenes the addition. The price table's growth to thirty-five priced models staled nothing, because the README names no priced model count. Nothing else in the README needed to change.

## Round 46: the docs surface accuracy sweep, second running (2026-11-25)

The documented surface beyond the README was audited claim by claim for the second time, in the pattern of Round 40. Forty checkable claims were audited and thirty-seven stand true, among them the invariant list running INV-1 through INV-93 with no gaps, the examples against the fixtures they describe, the plugin and skill surfaces against the server's seven tools, and the crate READMEs against the crates they head. Three claims were stale. The roadmap's closing section ended its coverage at Round 40, and it now carries the programme through Round 45 in the same append form Round 41 used. INV-48 said the plugin manifest names telorama at the workspace version, while the manifest stands at the plugin's own version of 0.1.0 by design and the packaging test pins exactly that, so the invariant's wording now matches its enforcement and the manifest does not move. The dossier's architecture section named the concept's crate list from before the build, and it now carries a closing note naming the five crates as built, in the treatment Round 40 gave its open risks item. Nothing else on the surface needed to change.

## Round 47: version 0.6.0 (2026-11-25)

All five crates move from version 0.5.0 to version 0.6.0, carrying the work landed since the Round 35 release. The price rows for the current model lines and INV-93 ship in this version, with the two new agent run scenes and their goldens, the viewer record regenerated so its engine version stamp reads 0.6.0, and the accuracy programme's corrections to the README, the invariants, the dossier, and the roadmap. The README names the new version and counts forty-six completed rounds under the standing convention. Cargo.lock stays untouched in the repository, as in every release, and the packaging route refreshes it on the runner. The publish record for this round follows when the crates stand live on the index.

## Round 47 publish record (2026-11-25)

All five crates stand live on the crates.io index at version 0.6.0, published in the settled order of scene, ingest, svg, the command line crate, and the MCP server, with no version yanked. The packaged files were built on the hosted packaging route from the merged main and staged through the packaging branch, and the index carries versions 0.1.0 through 0.6.0 for every crate. Documentation for the new version is served by docs.rs.

## Round 48: the scene reel recomposed (2026-11-25)

The viewer scene reel was recomposed from the record as it now stands, following the Round 25 recipe against the current committed record. The reel is assembled from that record's own renders and nothing else, the incident scene split at its panel boundary and the six session frames stepped whole, in H.264 at 1160 by 868 for thirty seconds. Its frames carry the price table date of 2026-11-25 that Round 44 stamped into them, and the record behind them carries the engine stamp of version 0.6.0 from Round 47, where the Round 25 reel showed the record as it stood before either change. No frame was retouched and no figure was redrawn.

## Round 49: MCP currency re-checked (2026-11-25)

The three facts under telorama and its Events watch were read again firsthand. MCP Apps stands where the applicability work left it, with SEP-1865 Final on the SEP index and the stable extension specification still at its 2026-01-26 revision, so the view contract telorama implements is unchanged and only a draft revision is in development. The Events extension also stands unchanged, still in Beta under the same extension identifier and the same three delivery modes, with no support yet in the official Rust SDK, so neither trigger in the watch record has fired and the watch continues. The core protocol has moved twice since the revision telorama negotiates, first to 2025-11-25 and then to 2026-07-28, which the versioning page now names as the current version and which rebuilds negotiation as a stateless per-request exchange in place of the initialize handshake. Nothing in that move breaks the older revision telorama speaks, which stands as a final and frozen specification with published backward compatibility guidance, and this round records the gap without closing it.

## Round 50: README and roadmap currency (2026-11-25)

The two surfaces the recent rounds staled were brought current in their established forms. The README's round count read forty-six with forty-nine complete, and it now reads forty-nine under the standing convention, the recount derived from fifty-two round headings less the three companion publish records for Rounds 6, 13, and 47. Its other claims from the release round stand, the five crates at version 0.6.0 on the index and the golden corpus at seventeen scenes with six pinned in both themes, and nothing else in it had staled. The roadmap's closing section now carries the programme through Round 49 in the same append form as the earlier extensions, recording the docs sweep and its corrections, the release, the recomposed reel, and the MCP re-check with the protocol gap it recorded. Nothing else needed to change.

## Round 51: the protocol revision move, designed (2026-11-25)

A design record for moving telorama-mcp to the current MCP protocol revision stands at docs/REVISION-DESIGN.md. It settles the target as dual-era service, answering the 2025-06-18 handshake and the 2026-07-28 per-request exchange from the one stateless handler, because the compatibility guidance promises handshake-era clients nothing from a modern-only server and this server keeps no session state that would make serving both eras costly. The era is chosen per message, discovery is implemented as the current specification requires of servers, the Apps bridge stays untouched because the server is never a party to it, and the seven tools, the triple contract, the stdio transport, and every other crate stand as fences the build does not cross. The build is sliced into three gated pieces and governed by a drafted INV-94, and the record states plainly that the move buys no Events surface, which stays behind its own triggers. Three questions are left open by name, the Apps draft now in development, the server side Apps advertisement in discovery, and the legacy era's retirement as an owner call. The build itself waits on the owner's lock.

## Round 53: invariant traceability repaired (2026-11-25)

The traceability audit's eight defects are repaired. Every enforcement line in docs/INVARIANTS.md now names tests that resolve in the tree, the dangling cross-reference and the suite claims the tree did not bear out are corrected to the suites and tests that carry the claims, and the generic lines whose tests the audit verified now name them. The one dead citation is resolved on the evidence of whether its behaviour was tested under another name or not at all, and the record of that resolution stands in the round's own report. The two process enforcements, publish order and the missing-docs lint, keep their form because the generic statement is their mechanism. No invariant's claim text changed.

## Round 54: OpenInference fable scene dark pin (2026-11-25)

The OpenInference fable scene gains its dark pin, the seventh of the seventeen scenes pinned in both themes, following the Round 38 pattern. The dark golden was generated on the hosted route from the scene as it stands, and the README's count moves with it. The corpus now stands at twenty-four goldens, seventeen light and seven dark.

## Round 55: GenAI current scene dark pin (2026-11-25)

The GenAI current scene gains its dark pin, the eighth of the seventeen scenes pinned in both themes, following the same pattern as the fable pin before it. The dark golden was generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus now stands at twenty-five goldens, seventeen light and eight dark.

## Round 56: docs surface accuracy sweep, third running (2026-11-25)

The docs surface was swept for the third time, seventy-nine checkable claims audited across the docs, the examples, the crate readmes, the plugin, and the web copy, with seventy-seven standing true. The one stale item was the roadmap's closing section ending at Round 49, and it now carries the programme through Round 55 in the same append form as the earlier extensions. One claim was left unverified by design rather than corrected, the dossier's rendering-host snapshot from the November research, which is dated framing in a concept document and belongs to a research pass of its own when it is refreshed. Nothing else on the documented surface had staled.

## Round 57: rendering-host snapshot refreshed (2026-11-25)

The dossier's rendering-host snapshot was re-derived firsthand from the vendors' current documentation, host by host. Every host the November snapshot named stands where it stood, and one host joined the set, Microsoft 365 Copilot, which renders MCP Apps inside Copilot chat. Goose's entry gains the desktop qualifier its own documentation supports, since its rendering is a desktop property and its CLI does not render. The snapshot now carries its refresh date inside the bullet. Two findings ride with the refresh and are recorded here rather than acted on. Anthropic's current guidance asks app authors to set a ui.domain on their resource in Claude's hashed format, tied to apps that run their own OAuth flow, which telorama does not, and the view stands verified rendered in Claude as built, so nothing is broken, but the guidance postdates the view and a local stdio server has no server URL to hash if it ever hardens into a requirement, so it joins the watch list. And the new host renders only from remote servers wired into declarative agents under OAuth or Entra sign-on, so it widens the set of rendering hosts without widening telorama's practical audience, which a local stdio server does not reach.

## Round 58: API scene dark pin (2026-11-25)

The API scene gains its dark pin, the ninth of the seventeen scenes pinned in both themes, following the same pattern as the GenAI current pin before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at twenty-six goldens, seventeen light and nine dark.

## Round 59: Heatmap scene dark pin (2026-11-25)

The heatmap scene gains its dark pin, the tenth of the seventeen scenes pinned in both themes, following the same pattern as the API pin before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at twenty-seven goldens, seventeen light and ten dark.

## Round 60: Service map scene dark pin (2026-11-25)

The service map scene gains its dark pin, the eleventh of the seventeen scenes pinned in both themes, following the same pattern as the heatmap pin before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at twenty-eight goldens, seventeen light and eleven dark.

## Round 61: Codex agent run scene dark pin (2026-11-25)

The Codex agent run scene gains its dark pin, the twelfth of the seventeen scenes pinned in both themes, following the agent run pattern of the fable and GenAI current pins before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at twenty-nine goldens, seventeen light and twelve dark.

## Round 62: Unpriced Codex agent run scene dark pin (2026-11-25)

The unpriced Codex agent run scene gains its dark pin, the thirteenth of the seventeen scenes pinned in both themes, following the agent run pattern of the Codex pin before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at thirty goldens, seventeen light and thirteen dark.

## Round 63: GenAI agent run scene dark pin (2026-11-25)

The GenAI agent run scene gains its dark pin, the fourteenth of the seventeen scenes pinned in both themes, following the agent run pattern of the GenAI current and Codex pins before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at thirty-one goldens, seventeen light and fourteen dark.

## Round 64: OpenInference agent run scene dark pin (2026-11-25)

The OpenInference agent run scene gains its dark pin, the fifteenth of the seventeen scenes pinned in both themes, following the agent run pattern of the GenAI and Codex pins before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at thirty-two goldens, seventeen light and fifteen dark.

## Round 65: Incident checkout scene dark pin (2026-11-25)

The incident checkout scene gains its dark pin, the sixteenth of the seventeen scenes pinned in both themes, following the incident pattern of the incident findings pin before it. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at thirty-three goldens, seventeen light and sixteen dark.

## Round 66: Agent state scene dark pin (2026-11-25)

The agent state scene gains its dark pin, the seventeenth and last of the seventeen scenes pinned in both themes, following the data pattern of the agent run pins before it, so every scene in the corpus now stands pinned in both themes. The dark golden will be generated on the hosted route from the scene as it stands, the README's count moves with it, and the INV-77 enumeration gains the new test's name. The corpus will stand at thirty-four goldens, seventeen light and seventeen dark.

## Round 67: README and roadmap currency (2026-11-25)

The two surfaces the dark pin sweep staled were brought current in their established forms. Seventy checkable claims were audited across the README and this roadmap, sixty-eight standing true and the two the intervening rounds had staled corrected. The README's round count read forty-nine with sixty-six complete, and it now reads sixty-six under the standing convention. Its other claims from the sweep stand, the five crates at version 0.6.0 on the index and the golden corpus at seventeen scenes with every one of them pinned in both themes, and nothing else in it had staled. The roadmap's closing section now carries the programme through Round 66 in the same append form as the earlier extensions, recording the third docs sweep and its one correction, the rendering-host refresh with the host that joined the rendering set, and the nine dark pins that completed the corpus at thirty-four goldens, seventeen light and seventeen dark. Nothing else needed to change.

## Round 68: Invariants traceability audit, second running (2026-11-25)

The Round 52 audit method was run a second time over the repaired tree, and the repaired state held. The census is complete at ninety-three invariants, every one carrying an enforcement line, and two hundred and ten genuine citations over one hundred and eighty-six distinct test names all resolve, with no dead citation anywhere. The growth from the repair's own verification is exact, the eleven dark golden tests the pin rounds added, each extending the INV-77 enumeration in its own round, and that enumeration names all seventeen dark tests the tree carries, no more and no fewer. The test Round 53 added under its cited name stands, and the reverse sweep counts two hundred and forty-three test functions with the uncited count unchanged from the post-repair state. No defects were found and no repair round is needed.

## Round 69: viewer export verified on the hosted route (2026-11-25)

The enlarged viewer page, whose local export verification had stood owed since the record grew past its first shape, was verified on the hosted route after the local route stayed blocked by this box's stale copy of the record file. A scratch branch workflow built the static export on a runner against the committed record as it stands on main, and the evidence was read from the run's own log. The record checked out at eighty-nine thousand six hundred and ninety-four bytes with its four sessions across the four dialects and the engine stamp of version 0.6.0, and the export built clean, its page standing at one hundred and seventy-eight thousand nine hundred and fifty bytes with every session's identifier and the engine stamp found inside it. The scratch branch was deleted with nothing landed from it. One finding rides with the verification, recorded rather than acted on. The web project carries no committed lockfile, so a clean install cannot run against it and the build resolves its dependencies fresh each time, where a committed lockfile would pin them.

## Round 70: the web lockfile committed (2026-11-25)

The finding Round 69 recorded is closed. The web project now carries its lockfile in the repository, generated from the project's own dependency set at the versions its ranges resolve to, so a clean install runs against it and the viewer build resolves the same dependency tree every time. The lockfile was proven on the hosted route before this entry landed, a scratch workflow on this round's own branch installing clean from the committed file and building the static export from that install. Nothing else in the web project changed.

## Round 71: the viewer reel is a recording of the page (2026-11-25)

The owed page capture is discharged, and the reel the README carries is now a recording of the viewer page itself rather than a composition of its frames. The capture ran on the hosted route, the export built clean from the committed lockfile, served locally on the runner, and recorded with Playwright at a 1280 by 800 viewport as the page was stepped through its own controls. Each session section opens on its last frame, so the recording walks each session back to its first frame with its Previous control, one step for each of the two sessions carrying two frames and none for the single frame sessions, exactly the stepping the record holds. The recording stands at 33.64 seconds, 1280 by 800, H.264, 894,089 bytes, and replaces the Round 48 composition at the same path. Earlier reels were assembled from the record's own rendered frames and were honest as far as they went, but the page itself is the surface a reader would meet, and it is the page the reel now shows.

## Round 72: README and roadmap currency, fourth running (2026-11-25)

The README and roadmap currency audit ran for the fourth time over main at the Round 71 merge. Eighty checkable claims stood across the two files, forty-six in the README and thirty-four in the roadmap, and seventy-seven stood true. Three had staled and this round corrects them. The README round count moves from sixty-six rounds to seventy-one under the standing convention. The README sentence for the scene reel now describes a recording of the viewer page itself, the Round 71 capture having replaced the composition of rendered frames the sentence was written for. The roadmap closing section gains the paragraph carrying the programme through Round 71, its account having ended at Round 66. Everything else the two files claim stood as written, the golden corpus at thirty-four with seventeen scenes pinned in both themes, all five crates at version 0.6.0, and the price table date at 2026-11-25.

## Round 73: docs surface accuracy sweep, fourth running (2026-11-25)

The docs surface was swept for the fourth time, eighty-eight checkable claims audited across the documents under docs on main at the Round 72 merge, with eighty-six standing true. Two claims had staled and this round corrects them. The fences of the Round 23 and Round 24 viewer design records each closed by saying the owed capture of the viewer page stays owed, and the Round 24 record said the same of the owed local export verification. Round 69 discharged the export verification and Round 71 discharged the capture, so both records now carry the discharge beside the fence they wrote. Everything else stood as written, the twenty design records reading as the round history they are, the dossier's rendering-host snapshot standing as refreshed in Round 57, the events watch standing on its Round 49 re-check with neither trigger fired, and the revision design standing with its build still waiting on the owner's lock. No code changed in this round.

## Round 74: examples and plugin surface accuracy audit (2026-11-25)

The examples and plugin surface was audited end to end for the first time, over main at the Round 73 merge. One hundred and eighteen checkable claims stood across the nine files under examples and plugin, and one hundred and sixteen stood true. One claim had staled and this round corrects it. The trace investigation walkthrough describes the checkout fixture as a frontend root, a checkout span, five service calls beneath it, and an email span, an enumeration of eight spans for a trace the same sentence calls nine-span, where the fixture in fact carries six service calls beneath the checkout span. The count now reads six. One further claim, the plugin README's direction to point Claude Code at the directory with its plugin flag, could not be settled from the tree, because it describes the host's behaviour rather than telorama's, and it stands as written. Everything else stood as written, every command line in the walkthroughs matching the CLI as built, every tool the skill names standing in the server's catalogue, the plugin manifest standing at its own version 0.1.0 under its packaging pin, and the walkthroughs' figures, the eleven log events, the two turns, and the first frame's tokens and cost among them, borne out by the fixtures and the golden corpus.

## Round 75: viewer page copy accuracy audit (2026-11-25)

The viewer page copy was audited for the first time, over main at the Round 74 merge. Thirty-nine checkable strings stood across the viewer's source and its readme, the masthead and tagline, the summary bar, the inspector and stepper labels, the session section introductions, the footer, and the build and record instructions in web/README.md, and every one stood true. The tagline and footer claim that the page recomputes no finding, figure, or hash is borne out by the source, which renders the record's findings, figures, and hashes verbatim and tallies only the delivered findings for the summary bar the design specifies. The session introductions print each record's dialect string verbatim with no display name map, each section opens on its last frame, and the stylesheet carries no transition or animation anywhere, so the copy's claim that the data never animates is borne out as well. The footer's corpus claims check against the record itself, all four finding kinds exercised between the incident and the sessions, and the incident heatmap anchored by no finding as the quiet case. No copy changed in this round.

## Round 76: CLI and MCP surface text accuracy audit (2026-11-25)

The user-facing and agent-facing text of the two binaries was audited for the first time, over main at the Round 75 merge. Fifty-nine checkable claims stood across the CLI's usage string, flag tables, error messages, and output lines and the MCP server's tool descriptions, parameter descriptions, required lists, and view resource entry, and fifty-five stood true. Four claims had staled and this round corrects them. The usage lines for session and frames bracketed every input flag as optional, although a session cannot be read without logs for the log dialects or spans for the span dialects, and the same string marks genuinely required arguments without brackets on its render, frames, and incident lines. Both lines now state the choice, logs with optional spans or spans alone. The incident tool's input schema carried the panel detail property set and so advertised a scene parameter and a panel index parameter the tool never reads, because the tool composes its scene from the trace export itself. Its schema now carries the data arguments alone. Everything else stood as written, every error message naming the condition that fires it, the session tools' spans description naming Claude Code alone as the fusion dialect because the Codex adapter merges nothing from spans, and the agent state description naming the three fields the parser requires of every entry.

## Round 77: crates.io and docs.rs surface accuracy audit (2026-11-26)

The published surface was audited for the first time, over main at the Round 76 merge. Sixty-seven checkable claims stood across the five crates on crates.io and docs.rs and the repository's own GitHub fields, and every one stood true. On crates.io each crate's newest and default version reads 0.6.0, matching the manifests on main, and every published description, license string, repository and homepage link, keyword set, and category set matches its manifest word for word. On docs.rs the 0.6.0 documentation stands for all five crates, and the landing text of the four library crates matches the doc comments on main, the scene crate performing no I/O and rendering nothing, the ingest crate reading a single export document or JSON Lines, the SVG crate's width rules standing as INV-92 states them, and the MCP crate's triple contract and app-only drill-down standing as the server implements them. The command line crate carries no library, and its docs.rs page renders its README, whose account of the commands, the cargo install line, the MCP server in its sister crate, and the Claude Code plugin package all check against the tree. The repository description on GitHub states what the software is and how it is used, and its homepage field stands unset, which states nothing. One wording was flagged rather than counted. The ingest crate's doc comment names the first milestone as the subject of its reading sentence, a label with no referent in the current product, while the formats it names are exactly the formats the parser reads. The claim stands true on its substance and the wording is left as written. No text changed in this round.

## Round 78: the 0.7.0 release (2026-11-26)

All five crates stand at version 0.7.0, published on crates.io in the settled order and verified live on the index, with docs.rs serving the new documentation for each. The release carries the Round 76 surface corrections to the published surface, the session and frames usage lines stating their input choice and the incident tool's schema naming only the arguments it reads. The ingest crate's doc comment loses the first milestone's name from its opening sentence, the wording the Round 77 audit flagged, so the published documentation now opens on the crate itself. Eight version literals moved across the workspace manifest, the five crate manifests, and the three dependency pins, the README's closing sentence carries the new version beside the round count of seventy-seven, and the viewer record was regenerated on the hosted route so its engine stamp reads 0.7.0 at an unchanged 89,694 bytes, its deep diff against the old record confined to that one field.

## Round 79: README and roadmap currency (2026-11-26)

The README and roadmap currency audit ran for the fifth time over main at the Round 78 record merge. One hundred checkable claims stood across the two files, sixty-seven in the README and thirty-three in the roadmap, and ninety-eight stood true. Two had staled and this round corrects them. The README round count moves from seventy-seven rounds to seventy-eight under the standing convention. The roadmap closing section gains the paragraph carrying the programme through Round 78, its account having ended at Round 71. Everything else the two files claim stood as written, all five crates published at version 0.7.0 and verified live on the index, the golden corpus at thirty-four with seventeen scenes pinned in both themes, and the price table date at 2026-11-25.

## Round 80: invariants traceability audit (2026-11-26)

The invariants traceability audit ran for the third time over main at the Round 79 merge and found no defects. Ninety-three invariants stand in the record and every one carries an enforcement line. Two hundred and ten genuine citations stand across those lines over one hundred and eighty-six distinct test names, and every cited name resolves to a test function in the tree, each standing in the test module its citation names. The reverse sweep counts two hundred and forty-three distinct test functions in the tree, fifty-seven of them cited by name nowhere, the same figures the second running returned at Round 68. The two invariants that name no test hold as written, one resting on the ceiling assertions in the scale tests named under its neighbour and one on the missing-docs deny gate standing at each library crate root, and the pricing module's unit tests and the golden corpus stand where the lines that cite them say they stand. No text changed in this round.

## Round 81: CLI surface completeness audit (2026-11-26)

The CLI surface completeness audit ran for the first time over main at the Round 80 merge, asking the inverse of the earlier accuracy audits, whether everything the surface offers is said in the README. The grammar carries eight commands and thirty flag forms across the five commands that take flags, and the MCP catalogue carries seven tools. Every tool stands plainly described in the README account of the server, and every command but one stands named with its working grammar. The schema command stood nowhere in the README although the validate account leans on the schema it prints, and two render flags stood unnamed, the state flag that feeds the agent state panel the README describes and the dialect flag a Codex session render needs. This round adds the three missing usage lines, a sentence giving the schema command its place beside validate, and the state report to the list of data the tool reads. The remaining gaps were judged honest omissions for a product account rather than a manual, the dialect flag on frames and on incident and the session inputs on incident among them, and they stand as written.

## Round 82: plugin skill completeness audit (2026-11-26)

The plugin skill completeness audit ran for the first time over main at the Round 81 merge, asking whether the skill teaches everything an agent needs rather than whether what it says is true. The skill names all seven MCP tools and all four session dialects, teaches seven of the nine panel families in its when-to-render guidance, and walks through the validate-first, inspect-first, and incident-first workflows. Three gaps were worth closing and this round closes them. The sankey and agent state panels stood nowhere in the skill although both render and the state panel is the plugin README's own picture, and the skill now gives each its paragraph, the state paragraph naming the agent_state argument that carries the report into a render. The render_scene bullet now says the data rides beside the spec as arguments, a mapping the skill had left to the tool schema alone. The honesty section now opens with the two authoring duties it had stated only as properties of the render, the required headline and the exact query or trace id behind every data entry, and the hosts section names the CLI's schema command beside render-html. The rest was judged fine to leave, the CLI command set beyond that one command, the dark theme, narrative items, and verdict thresholds among them, because the skill teaches judgment and the tool schemas carry the mechanics.

## Round 83: dialect and price currency re-checked (2026-11-26)

The dialect and price currency check ran for the second time over the price table and the four dialect adapters. Every figure in every priced row was re-read against the vendor's own pricing pages on 2026-11-26 and no figure had drifted, so the table stands unchanged and its as-of date stays at 2026-11-25 under the rule that the date advances only when the contents change. The vendor pricing page now publishes rates for GPT-6.1 Terra (1.00 input, 0.05 cached input, 5.00 output) and for GPT-6 Luna (0.10 input, 0.01 cached input, 0.50 output), in dollars per million tokens, and both ids still carry no row because no committed fixture names either one, as INV-93 requires. The bare gpt-5.1-mini id remains unpublished by the vendor and its row stays on the Round 42 rationale, with the corroboration strengthened because gpt-5-mini and gpt-5.1-codex-mini now both stand on the vendor's own model pages at the row's figures. The GenAI conventions repository cut its first standalone release, v0.1.0, on 2026-11-21, two days before the pin commit, and thirteen commits stand past the pin, and none of that movement renames or removes an attribute the adapter reads, the registry read set standing identical at the pin and at the current head, so the dialect stands current, and the pin header in the adapter is corrected in this round beside this entry, the earlier record having stated that the repository had cut no versioned release. No public source for the Claude Code, Codex, or OpenInference shapes shows a rename or a removal in anything those adapters read, and the Claude Code api_refusal event remains the one named candidate whose fields no live export has yet verified.

## Round 84: examples completeness audit (2026-11-26)

The examples completeness audit ran for the first time over main at the Round 83 merge, asking whether the worked examples walk through everything the product teaches rather than whether what they say is true. The four walkthroughs were read against the thirteen workflows the README and the skill teach and against the nine panel families. Nine workflows stand worked end to end and six of the nine panel families stand worked, the service map and the heatmap through the incident composition. Four gaps were worth closing and this round closes them. The schema command stood nowhere in the examples and the trace investigation now prints it beside its validate step. The flame and sankey panels stood nowhere although both render over the checkout fixtures, and the trace investigation now renders the profile and stages the flow. The agent state panel stood nowhere although its scene, its report fixture, and its golden all stand in the tree, and the agent session walkthrough now puts a state report in a scene. Three gaps were judged fine to leave. The standalone HTML form stays a single sentence because that sentence is the whole workflow, the panel drill-down stays unworked because it is app-only and happens inside a host view rather than at the command line, and the incident template's fifth panel stays a mention because no committed session export was recorded inside the checkout window to pair with the trace.

## Round 85: gallery freshness verified on the hosted route (2026-11-26)

The gallery freshness verification ran for the first time over main at the Round 84 merge. The gallery page is a generated artifact, assembled from the scene fixtures and their renders, and a scratch branch regenerated it on the hosted route under UPDATE_GALLERY=1 and compared the result against the committed page. The regeneration produced no change to commit and the committed page stands byte-identical to a fresh generation at 147,612 bytes, so the gallery is fresh. The generating test asserts the same equality on every gate run, and this round stands as the independent regeneration proof beside that assertion, on the pattern the viewer export verification set in Round 69.

## Round 86: README and roadmap currency (2026-11-26)

The README and roadmap currency audit ran for the sixth time over main at the Round 85 merge. One hundred and sixty-nine checkable claims stood across the two files, ninety-eight in the README and seventy-one in the roadmap, and one hundred and sixty-seven stood true. Two had staled and this round corrects them. The README round count moves from seventy-eight rounds to eighty-five under the standing convention. The roadmap closing section gains the paragraph carrying the programme through Round 85, its account having ended at Round 78. Everything else the two files claim stood as written, all five crates published at version 0.7.0, the golden corpus at thirty-four with seventeen scenes pinned in both themes, the price table date at 2026-11-25, and the three usage lines the completeness round added standing in forms the grammar accepts.

## Round 87: viewer reel currency checked (2026-11-26)

The viewer reel was checked for currency against the page it records for the first time, over main at the Round 86 merge. The record on main differs from the record the Round 71 capture was recorded against in exactly one field, the engine stamp moving from 0.6.0 to 0.7.0 in the Round 78 regeneration at an unchanged 89,694 bytes, and every other field the page renders stands byte-identical, the four sessions and their 2, 2, 1, 1 frame shape among them. No viewer source has changed since before the capture either, every file under web/app carrying its last change behind the Round 71 merge. The one changed field is nevertheless a rendered one. The masthead provenance line prints the record's generator, engine stamp, scene spec version, and fixture list, so the page today reads engine 0.7.0 where the reel's opening frames read engine 0.6.0, and the reel is stale in that line alone. Everything past the masthead, the incident section and each session section with its frames, findings, and hash lines, matches the page by construction. A re-capture under the Round 71 recipe, the hosted export served locally and recorded at 1280 by 800 while each session steps back through its Previous control, would change nothing but that stamp, and the reel stands owed that re-capture.

## Round 88: viewer reel re-captured (2026-11-26)

The viewer reel was re-captured on the hosted route under the Round 71 recipe, discharging the debt Round 87 recorded. The export built from the committed lockfile, was served locally on the runner, and was recorded at 1280 by 800 while each session stepped back through its Previous control, one step for each two-frame session and none for the single-frame sessions. The new recording is H.264 at 1280 by 800 and 41.08 seconds, 890,153 bytes against the old reel's 894,089, and a frame drawn from its opening seconds shows the masthead provenance line reading engine 0.7.0, the one stale line now correct. The standing rule from Round 87 is restated for the release rounds ahead of this one. A release that regenerates the viewer record owes the reel a re-capture in the same pass whenever the record's engine stamp moves, because the stamp is printed on the page the reel records.

## Round 89: docs surface accuracy sweep (2026-11-26)

The docs surface accuracy sweep ran for the fifth time over main at the Round 88 merge. One hundred and forty-one checkable claims stood across the thirty-five text documents under docs/, the generated gallery page noted and not counted, and one hundred and thirty-nine stood true. Two had staled, and both carried the one false premise the Round 83 currency check had already corrected in the adapter's pin header and in this record. The Round 3 currency design record states that the GenAI conventions work continues in a dedicated repository with no release yet, and the concept dossier states the conventions stand with no release to pin, and the repository in fact cut its first release, v0.1.0, on 2026-11-21, two days before the currency record settled and five days after the dossier was written. Both records keep their sentences and gain the correction beside them in this round, the discharge standing next to the claim the way the Round 73 corrections left theirs. Everything else stood as written, the events watch on its dated re-check, the revision design on a server that still negotiates the revision it names, and the roadmap's closing account current through Round 85 with its two standing waits, the revision build on the owner's lock and the GPT-6.2 pair on a real export, both still true.

## Round 90: MCP currency re-checked (2026-11-26)

The MCP facts under telorama and its Events watch were read again firsthand, one day after Round 49 read them, and nothing has moved. MCP Apps stands where it stood, with SEP-1865 Final in the specification repository and the stable extension specification in the ext-apps repository still at its 2026-01-26 revision, its text last committed on the day of its release, so the view contract telorama implements and the protocol version its view negotiates are unchanged and only a draft revision is in development. The Events extension also stands unchanged, Final as a SEP and still in Beta in the official documentation and in its own repository, whose newest commit predates Round 49, with no support yet in the official Rust SDK, so neither trigger in the watch record has fired and the watch continues on its stated terms. The core protocol has not moved either, the versioning page still naming 2026-07-28 as the current version over the same five dated revisions, and the revision telorama negotiates still standing final and frozen at 2025-06-18 in its server constant. Anthropic's ui.domain guidance stands as Round 57 recorded it, a hashed value an author sets from a server URL, still not a condition of rendering and still unavailable to a local stdio server with no URL to hash, so the watch item rides unchanged. This round records the re-check and changes nothing.

## Round 91: invariants traceability audit (2026-11-26)

The invariants traceability audit ran for the fourth time over main at the Round 90 merge and found the traceability clean again, its figures identical to the third running in every dimension. Ninety-three invariants stand in the file and every one carries its enforcement line. The enforcement lines make two hundred and ten genuine citations over one hundred and eighty-six distinct test names, and every citation resolves in the module its line names, the reverse sweep finding two hundred and forty-three distinct test functions across the workspace with fifty-seven uncited by name. No code has landed since the third running beyond a doc comment in the GenAI adapter, and the audit confirms nothing moved underneath the record in the interval. No defects were found and no repair round is charged.

## Round 92: examples and plugin surface accuracy audit (2026-11-26)

The examples and plugin surface was audited end to end for the second time, over main at the Round 91 merge. One hundred and twenty-four checkable claims stood across the nine files under examples and plugin, and one hundred and twenty-three stood true, with none staled. The single claim that could not be settled from the tree is the one the first running left standing, the plugin README's direction to point Claude Code at the directory with its plugin flag, which describes the host's behaviour rather than telorama's and stands as written, and the claim the first running corrected, the six service calls beneath the checkout span, stands true against the fixture. Everything the two intervening rounds added was verified with the rest of the surface, the skill's render argument mapping, sankey and agent state paragraphs, schema sentence, and two authoring duties from Round 82, and the trace investigation's schema step, profile rendering, and flow staging with the agent session walkthrough's state report section from Round 84, and the figures in the new material, the profile's 970 samples at 63.9 percent and the flow's staging of one, one, six, and one spans by depth among them, are borne out by the fixtures and the golden corpus. The total stands six above the first running's one hundred and eighteen, the additions carrying twenty-five new claims and this running counting compound enumerations once rather than part by part, the plugin README's seven tools and the frame payload's six fields among them, where the first running counted their parts.

## Round 93: viewer page copy accuracy audit (2026-11-26)

The viewer page copy was audited end to end for the second time, over main at the Round 92 merge. Fifty-six checkable strings stood across the viewer sources and every one stood true, with none staled and none unsettleable. The masthead provenance line renders the record's own fields and its engine stamp reads 0.7.0, the value the Round 78 release regenerated the record with, and the summary counts the page prints beside it, one each for errors, latency, and wait and four for cost, are the counts its stated scope produces from the record, the incident investigation and each session's final frame. The page sources themselves have not changed since the first running read them, their last change landing in Round 23, and the only movement anywhere in the surface since is the record's engine stamp, so the higher total against the first running's thirty-nine is enumeration alone, this running counting the kind labels, panel labels, summary counts, and single prose sentences as separate strings where the first counted copy blocks. The one judgment the first running recorded stands as it stood, the claim that nothing on the page is computed in the browser holding in the governing design's own sense, the page displaying delivered artifacts and counting delivered findings without recomputing any finding, figure, or hash.

## Round 94: CLI and MCP surface text accuracy audit (2026-11-26)

The user-facing and agent-facing text of the two binaries was audited for the second time, over main at the Round 93 merge. Seventy-four checkable claims stood across the CLI's usage lines, error messages, and output lines and the MCP server's tool descriptions, parameter descriptions, required lists, view links, and view resource entry, and every one stood true, with none staled and none unsettleable. The four claims the first running found staled stand corrected as that round left them, the session and frames usage lines stating their input choice of logs with optional spans or spans alone, and the incident tool's schema carrying the data arguments alone while the panel detail tool keeps its scene and panel index beside them. Everything else the first running verified stands as written, every error message naming the condition that fires it, the session tools' spans description naming Claude Code alone as the fusion dialect because the Codex adapter merges nothing from spans, and the agent state description naming the three fields the parser requires of every entry. The total stands fifteen above the first running's fifty-nine on enumeration alone, this running counting each error message, each output line, each view entry field, and each view link as its own claim where the first running grouped the surface more coarsely, and the only text that has moved in the interval is the four corrections themselves and one doc comment. No text changed in this round.

## Round 95: crates.io and docs.rs surface accuracy audit (2026-11-26)

The published surface was audited end to end for the second time, over main at the Round 94 merge and the live crates at version 0.7.0. Sixty-seven checkable claims stood across the five crates and every one stood true, the same total the first running found against 0.6.0. Every published description, license string, repository and homepage link, keyword set, and category set on crates.io matches its manifest word for word, the documentation field stands unset in the published record as the manifests leave it, the published rust version reads 1.88 as the workspace declares, and 0.7.0 stands newest and unyanked for all five crates. Docs.rs serves 0.7.0 for all five, presents each description and the repository link, and renders the crate documentation for the four library crates while the telorama binary crate presents its crate page alone as its shape requires, and the ingest crate's documentation carries the corrected wording the 0.7.0 release landed after the first running flagged the old line. The GitHub repository description stands as the first running verified it and its homepage field stands unset. Nothing on the published surface has staled and no new release is owed.

## Round 96: showcase media currency audit (2026-11-26)

The four still media artifacts were audited for currency for the first time, over main at the Round 95 merge, their depicted content read directly and compared against the golden corpus and the viewer record. All four stand current. The agent run still and the agent state still match their golden scenes string for string, and their dimensions are exactly those scenes scaled to the showcase width of 1160, and the incident still matches the findings variant of the incident golden in every figure, its headline, both findings in its strip, the service map and error row figures, the waterfall spans, and the heatmap bucket counts among them. The frames GIF carries the demo session's two frames with every figure, finding, and label matching the record's frame scenes, its third frame a trailing hold repeating the second, and the one textual difference anywhere in the four artifacts is a provenance line, the GIF's frames reading generated by telorama-cli where the record's read telorama-viewer, which is the correct stamp for each production route, the CLI stamping its frames output with its own name and the record assembly stamping its own. No artifact needs re-capture and no media changed in this round.

## Round 97: docs surface accuracy sweep (2026-11-26)

The docs surface accuracy sweep ran for the sixth time over main at the Round 96 merge. One hundred and forty-six checkable claims stood across the thirty-five text documents under docs and every one stood true, with none staled and none unverifiable. The two claims the fifth running found staled, the GenAI conventions release premise in the currency design record and in the dossier, stand corrected as that round left them, each original sentence keeping its place with the correction beside it, and the corrections' own assertions check against the record, the conventions repository's first release standing on 2026-11-21 and the adapter's pin unchanged. The whole surface and every ground truth it rests on stand byte-identical to the fifth running's audit state apart from this record itself, and the external facts the documents rely on were read again in the rounds between, the MCP facts in Round 90 and the price and dialect facts in Round 83, with nothing moved under any of them. The total stands five above the fifth running's one hundred and forty-one, the two correction paragraphs carrying assertions of their own and the roadmap's closing account read at a finer grain, and the gallery page stands as the generated artifact it is, proven byte-identical to a fresh generation in Round 85 and not counted here.

## Round 98: dialect and price currency re-checked (2026-11-26)

The dialect and price currency check ran for the third time. Every figure in the price table was read again against the vendors' own published pages and no figure in any of the nineteen rows has drifted, so the table stands unchanged and its as-of date stays at 2026-11-25 under the rule that the date moves only when the contents do. The deprecations the second running recorded stand exactly as recorded, with gpt-5.1, gpt-5.3-codex, and gpt-5.4-nano still leaving the OpenAI API on 1 April 2027. The same page carries colour the second running did not record that touches priced ids without moving any figure, with gpt-5, gpt-5-mini, and gpt-5-nano shutting down on 11 December 2026, a January 2027 retirement scoped to gpt-5.2, gpt-5.4, and gpt-5.4-mini under EU data residency at the fast tiers, and codex-mini-latest standing shut down on its model page with its published price unchanged. One gated fact has half moved. OpenAI's pricing page now publishes the GPT-6.2 pair, gpt-6.2-astra at 8.00 input, 0.40 cached input, and 40.00 output, and gpt-6.2-sol at 2.00, 0.10, and 10.00, so a vendor price now stands behind every figure for both ids, while bare gpt-6.2 still carries no rate of its own, claude-haiku-5.5 still publishes only its tiered rate, and gpt-5.4-pro still publishes no cached rate. The pair stays rowless all the same, because INV-93 asks for a committed fixture naming the id as well, and that half of the gate still waits on a real Codex export of a GPT-6.2 run. The GPT-6.1 Terra rate at 1.00, 0.05, and 5.00 and the GPT-6 Luna rate at 0.10, 0.01, and 0.50 stand re-confirmed from the vendor's pages, both still rowless for want of a fixture. The GenAI dialect stands current in the strongest form the check can show, the conventions repository standing on the same head the second running read, the same thirteen commits past the pin, no release past v0.1.0, and every attribute name and operation value the adapter reads standing in the registry at that head, and nothing the OpenInference or Codex adapters read has moved. The Claude Code api_refusal event stands on firmer public ground than the second running found, with Anthropic's monitoring documentation now naming the event and its attribute set published and growing, though its fields still stand unverified against an export of the project's own and the adapter's handling of the event as a failed call with no tokens stands as built. No text changed in this round.

## Round 99: README and roadmap currency (2026-11-26)

The README and roadmap currency audit ran for the seventh time over main at the Round 98 merge. One hundred and seventy-seven checkable claims stood across the two files, ninety-eight in the README and seventy-nine in the roadmap, and one hundred and seventy-five stood true. Two had staled and this round corrects them. The README round count moves from eighty-five rounds to ninety-eight under the standing convention. The roadmap closing section gains the paragraph carrying the programme through Round 98, its account having ended at Round 85. Everything else the two files claim stood as written, all five crates published at version 0.7.0, the golden corpus at thirty-four with seventeen scenes pinned in both themes, the viewer reel standing at the Round 88 re-capture of 890,153 bytes, and the price table's date holding at 2026-11-25 with no figure drifted in the third currency running. The total stands eight above the sixth running's one hundred and sixty-nine, the paragraph that running added carrying assertions of its own into the roadmap's census.

## Round 100: the programme at one hundred rounds (2026-11-26)

The closing full-state audit reconciled the programme over main at the Round 99 merge and found the record accurate in every particular it measures. Ninety-nine rounds stand closed before this one and this audit is the hundredth, the count reconciling exactly from the record's own headings, ninety-two in the current form and six in the older comma form, with Round 52 carried inside the Round 53 entry and no heading of its own. The tree carries one hundred and eighty-seven files, the golden corpus stands at thirty-four with seventeen scenes pinned in both themes and no dark orphan, and fifteen scene fixtures stand beneath them. The viewer record stands at 89,694 bytes with its engine stamp at version 0.7.0 and the reel at 890,153 bytes. The invariants file ends at INV-93, its traceability last audited clean in Round 91, and INV-94 stands drafted in the revision design and nowhere else. All five crates stand published at version 0.7.0, newest and unyanked, with docs.rs serving that version, and the checks on the main head stand green. The price table's date holds at 2026-11-25 with its five rowless pins standing. The programme closes its hundredth round with its open items named plainly and carried forward, each one gated on something outside the repository rather than on unfinished work inside it. The GPT-6.2 price rows wait on a real Codex export of a GPT-6.2 run, the pair priced on its vendor's page since Round 98 and held only by the fixture half of INV-93. The protocol revision build stands designed in full and unbuilt, waiting on the owner's lock. The api_refusal event's fields stand unverified against a live export of the project's own. The Events watch stands with neither trigger fired, and the ui.domain guidance stands as a watch item and not a rendering condition. Everything the audits measure stands verified.

## Round 101: README and roadmap currency (2026-11-26)

The README and roadmap currency audit ran for the eighth time over main at the Round 100 merge, the first running since the hundred-round programme closed. One hundred and ninety-two checkable claims stood across the two files, ninety-eight in the README and ninety-four in the roadmap, and one hundred and ninety stood true. Two had staled and this round corrects them. The README round count moves from ninety-eight rounds to one hundred under the standing convention. The roadmap closing section gains the paragraph carrying the programme through Round 100, its account having ended at Round 98. Everything else the two files claim stood as written, all five crates published at version 0.7.0, the golden corpus at thirty-four with seventeen scenes pinned in both themes, the viewer reel standing at the Round 88 re-capture of 890,153 bytes, and the price table's date holding at 2026-11-25 with no figure drifted in the third currency running. The total stands fifteen above the seventh running's one hundred and seventy-seven, the paragraph that running added carrying assertions of its own into the roadmap's census.

## Round 102: MCP currency re-checked (2026-11-26)

The MCP currency check ran for the third time and every fact the second running read was read again firsthand. SEP-1865 stands where it stood, the MCP Apps extension spec still stable at its 2026-01-26 revision with that text untouched since its release day commit, and the telorama view still negotiates that same revision in its ui/initialize handshake. SEP-3415 stands where it stood, Final in the Extensions Track with the Events extension still at Beta under io.modelcontextprotocol/events, its normative text still under specification/beta/events in the ext-events repository whose newest commit still predates the first running of this check, and still no support for the extension in the official Rust SDK. Neither watch trigger in the Events record has fired, telorama-mcp fronts no upstream system of its own and the extension has not left Beta, so the watch continues on its stated terms and no append to the watch record is owed. The core revision set stands unchanged, 2026-07-28 current and 2025-06-18 final and frozen, and the server constant in telorama-mcp still names 2025-06-18. Anthropic's ui.domain guidance for Claude stands unchanged in wording and standing, a sandbox origin computed from a server URL that a local stdio server does not have, and still not a condition the documentation places on rendering. No text changed in this round.

## Round 103: invariants traceability audit (2026-11-27)

The invariants traceability audit ran for the fifth time over main at the Round 102 merge and found the traceability clean again. The invariants file carries ninety-three invariants numbered INV-1 through INV-93 with no gap and no duplicate, and every one carries an enforcement line. The enforcement lines hold two hundred and ten genuine citations over one hundred and eighty-six distinct test names, and every cited name resolves to a test function of that exact name in the module its citation names, the pricing and renderer unit tests included. The reverse sweep counts two hundred and forty-three distinct test functions across the five crates, fifty-seven of them uncited by name. Every figure stands identical to the fourth running in Round 91, and the tree comparison behind that identity is mechanical, with DECISIONS.md, README.md, and the roadmap the only paths changed since the Round 91 merge and every Rust file and the invariants file itself blob-identical. Two invariants carry no test citation by design and stand as they stood, INV-70 enforcing through the scale tests named under INV-69 and INV-80 through the missing_docs deny gate at each library crate root. No defect was found and no repair round is owed.

## Round 104: examples and plugin surface accuracy audit (2026-11-27)

The examples and plugin surface accuracy audit ran for the third time over main at the Round 103 merge. One hundred and sixty checkable claims stood across the nine files under examples and plugin, and one hundred and fifty-nine stood true, with none staled and one unverifiable. The unverifiable claim is the plugin README's direction to point Claude Code at the directory with its plugin flag, which describes the host's behaviour and stands as written, as it stood in the second running. Every figure the walkthroughs print was read again against its fixture or golden, the nine-span checkout trace with its depth staging of one, one, six, and one, the profile's seven stacks and 970 samples with the payment charge subtree at 620 and 63.9 percent, the demo session's eleven log events and six spans with two merged, the state report's two entries, and the first frame's 3000 input tokens, 750 output tokens, and $0.0270. The Round 82 skill additions and the Round 84 example additions all stand as landed and read true. The total stands thirty-six above the second running's one hundred and twenty-four on enumeration granularity alone, this running counting each independently falsifiable assertion where the second folded compound enumerations into one, and the surface itself has not moved, every file under examples and plugin standing byte-identical to the second running's audit state and every ground truth it rests on unchanged. No text changed in this round.

## Round 105: viewer page copy accuracy audit (2026-11-27)

The viewer page copy accuracy audit ran for the third time over main at the Round 104 merge and found the page copy clean again. Fifty-six checkable strings stood across the page sources, thirty-eight in the viewer component, two in the layout metadata, and sixteen in the web README, with the page component and the stylesheet contributing none, and all fifty-six stood true with none staled and none unverifiable. The summary counts were recomputed from the committed record over the page's stated scope of the incident and each session's final frame and match what the page prints, one error finding, one latency finding, one wait finding, and four cost findings, and the masthead provenance line reads engine 0.7.0 from the record's own stamp. Every audited file and the record itself stand byte-identical to their state at the second running and at the 0.7.0 release, so the census and its verdicts carry over on proven ground rather than on assumption. The standing judgment from the first running stands unchanged, that the web README's claim that nothing on the page is computed in the browser reads true in the governing design's own sense. No text changed in this round.

## Round 106: CLI and MCP surface text accuracy audit (2026-11-27)

The CLI and MCP surface text accuracy audit ran for the third time over main at the Round 105 merge and found the surface text clean again. Seventy-four checkable claims stood across the two files, thirty-three in the command line tool and forty-one in the MCP server, and all seventy-four stood true with none staled and none unverifiable. The four Round 76 corrections stand verbatim as landed, the session and frames usage lines in their corrected alternation form, the incident tool's schema carrying the shared data arguments alone with traces required, and the panel detail tool keeping its own properties with scene and panel index required. Neither surface file has moved since the second running, both standing blob-identical to their state at the Round 94 merge with only the decisions record, the README, and the roadmap changed anywhere in the tree, so the census and its verdicts carry over on proven ground rather than on assumption. No text changed in this round.

## Round 107: crates.io and docs.rs surface accuracy audit (2026-11-27)

The crates.io and docs.rs surface accuracy audit ran for the third time over the published 0.7.0 surface and main at the Round 106 merge and found the published surface clean again. Sixty-seven checkable claims stood across the three surfaces, forty-five on crates.io at nine per crate, twenty on docs.rs at four per crate, and two in the GitHub repository fields, and all sixty-seven stood true with none staled and none unverifiable. Every published description, license string, repository and homepage link, keyword set, and category set matches its manifest on main word for word, the documentation field stands unset as the manifests leave it, the published rust version reads 1.88, and 0.7.0 stands newest and default and unyanked for all five crates over seven published versions each. Docs.rs serves 0.7.0 for all five, rendering the four library crates' documentation and the telorama binary's crate page alone, and the ingest crate's lead line reads in its corrected form. The GitHub repository description stands as recorded and its homepage field stands unset. The totals and the verdict stand identical to the second running in Round 95. No text changed in this round.

## Round 108: docs surface accuracy sweep (2026-11-27)

The docs surface accuracy sweep ran for the seventh time over main at the Round 107 merge and found the documented surface clean again. One hundred and eighty-six checkable claims stood across the thirty-five text documents under docs, the thirty-four markdown files and the gallery shell, with the generated gallery page noted and not counted, and all one hundred and eighty-six stood true with none staled and none unverifiable. The two correction paragraphs the fifth running landed stand as written and read true, in the currency design record and in the dossier, each recording that the GenAI conventions repository cut v0.1.0 on 2026-11-21. Every load-bearing figure was read again against its ground truth in the tree, the golden corpus at thirty-four with seventeen scenes pinned in both themes, the fifteen scene fixtures, the viewer record at 89,694 bytes with its engine stamp at 0.7.0, the price table date at 2026-11-25, the nesting budget of 512 in the scene crate, the 48-level flame limit, and the invariants file ending at INV-93 with INV-94 drafted in the revision design alone. The total stands forty above the sixth running's one hundred and forty-six, the rise coming from the roadmap's closing account, which gained two paragraphs in the record rounds since that running and which this running counts at a finer grain, while every other document under docs stands byte-identical to the sixth running's audit state and the surface's ground truth has not moved. No text changed in this round.

## Round 109: dialect and price currency re-checked (2026-11-27)

The dialect and price currency check ran for the fourth time over main at the Round 108 merge. No price figure drifted. Every figure in every priced row of the table in the ingest crate, nineteen arms carrying thirty-five priced ids, was read again against the vendors' own pages on 2026-11-27 and every row judged current, so the table's contents stand unchanged and its as-of date stays at 2026-11-25 under the standing rule. The GPT-6.2 pair stands as the third running left it, OpenAI now publishing gpt-6.2-astra at 8.00 input, 0.40 cached input, and 40.00 output and gpt-6.2-sol at 2.00, 0.10, and 10.00, so the pair's price condition under INV-93 stands met and the fixture condition alone holds the two rows, no committed fixture naming either id and no real Codex export of a GPT-6.2 run having reached this seat. Bare gpt-6.2 still carries no published rate, claude-haiku-5.5 still publishes tiered rates only, and gpt-5.4-pro still publishes no cached input rate, so all three stay rowless, and GPT-6.1 Terra at 1.00, 0.05, and 5.00 and GPT-6 Luna at 0.10, 0.01, and 0.50 stand re-confirmed as published and rowless for lack of a fixture naming either. The GenAI dialect stands current in its strongest form, the conventions repository standing on the identical head the third running read, thirteen commits past the pin and no release past v0.1.0, so no commit since that reading touches the adapter's read set. The Claude Code, Codex, and OpenInference read sets stand unmoved, and the Claude Code api_refusal event's fields still stand unverified against a live export of this project's own, its handling as a failed call with no tokens standing as built. No text changed in this round.
