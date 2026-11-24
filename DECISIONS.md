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
