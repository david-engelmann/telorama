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
