# The telorama roadmap

This roadmap runs from the state the publish left behind: milestones
1 through 5 landed and audited, the investigation layer, the agent
state panel, and the frame stream in place, 121 tests green on the
hosted gate, and all five crates published on crates.io at version
0.1.0. The fences from the live design bind every round: no
datastore, no query language, no dashboard editor, no alerting, no
hosted renderer, no streaming server.

## Round 1: Close the one declared gap, the sankey renderer

Sankey panels are specified and validated but have no renderer yet,
and the README states it in public. A launch surface does not carry
a declared hole. The design stands in docs/SANKEY-DESIGN.md with
invariants INV-58 to INV-61. The README sentence comes out in the
same landing that closes the gap.

## Round 2: The gallery

A static gallery page in the repo shows every scene family rendered
from the golden fixtures, each with its findings strip and its
provenance line, the incident scene and the frame stream at its
head. Static HTML only, every picture the renderer's own output,
assembled in CI the way the README media was, and linked from the
README's first screen.

## Round 3: Dialect and price-table currency

The moat is dialect-normalizing ingest and it decays on a schedule.
The GenAI semantic conventions keep moving, the Claude Code and
Codex exports evolve, and the renderer price table carries an
as-of date. This round re-derives each dialect against its current
published conventions, adds fixtures for any new shape before
supporting it, refreshes the price table with its date advanced,
and records the pass in DECISIONS.

## Round 4: Scale and performance proof

The fixtures are realistic but small. This round builds the large
shapes: a ten-thousand-span trace, a five-hundred-turn session, a
hundred-thousand-sample profile, each as a checked-in fixture with
a CI timing budget the gate enforces. Any layout or derivation
that degrades past linear gets fixed in the round, with the bound
written into INVARIANTS.

## Round 5: A second theme, drawn for dark hosts

Every scene renders on white, and chat hosts increasingly sit on
dark surfaces. This round adds a dark theme as a scene-level
choice: the same layout, the same figures, a palette derived from
Okabe-Ito with contrast checked per pair, byte-determinism per
theme, goldens for both themes on the flagship scenes. The light
theme stays the default and its bytes do not change.

## Round 6: The registry finish

Each crate page on crates.io carries its weight: readme rendering
verified per crate, docs.rs builds checked for all five crates
with every public item documented, examples in the crate docs that
compile as doc tests. A follow-up release carries whatever Rounds
1 to 5 changed, in the same dependency order as 0.1.0.

## After the roadmap: Round 7, the documented surface

The six rounds above are complete. Round 7 takes up
the two items the registry round left named: the
public surface documented field by field under a
permanent missing-docs gate, and findings parity
between the CLI and the other surfaces. Design
record docs/DOCS-DESIGN.md. Named candidates
beyond it: Claude Code refusal events,
dated-snapshot model ids, and cycle-aware
service-map layering.

## Round 8: hostile inputs

Round 8 takes up the three candidates named
above, which share one shape: real exports the
code handles badly. A cyclic trace hangs the
waterfall's critical path and inflates the
service map's layers, a refused model call
vanishes from the session record, and a dated
snapshot model id silently goes unpriced. Design
record docs/HOSTILE-DESIGN.md.

## Round 9: parity completion

Round 9 closes the two asymmetries the Round 7
reviews named and left standing: the CLI
incident command composes over its traces alone
while the MCP tool composes over its full
input, and the CLI's two input flags can count
one span twice. Design record
docs/PARITY-DESIGN.md.

## Round 10: the named leftovers

Round 10 takes the three leftovers Rounds 8
and 9 recorded without fixing: the MCP tools
do not dedupe their input where the CLI now
does, service_layers can panic on a hand-built
graph whose edge names an absent service, and
the flame builder's recursion can overflow on
a very deep acyclic trace. Design record
docs/LEFTOVERS-DESIGN.md.

## Round 11: depth, finished

Round 11 takes the two depth paths Round 10
named: the folded-stack builder still inserts
and freezes recursively, and the MCP panel
summary and detail paths still drop the flame
trees they own. One shared dismantle in
telorama-scene, an iterative folded-stack
builder, and the INV-86 exception clause
retires. Design record docs/DEPTH-DESIGN.md.

## Round 12: the agent tree

Round 12 takes the two agent-tree paths Round
11's sweep named: the dialect session
builder's emit recurses over an acyclic span
tree, and the agent-run layout walk recurses
over a run's nesting, with the run trees' own
drops on the same paths. Both walks iterate,
and one nesting budget of 512, enforced where
nesting is built, bounds every run tree.
Design record docs/AGENT-TREE-DESIGN.md.

## Standing lanes between rounds

- The A2UI watch in docs/A2UI-SPIKE.md stays a watch item,
  reviewed once per round against the spec's movement, never
  built on spec.
- One check per round that the GenAI conventions and the MCP Apps
  spec have not moved under the adapters.
- Every new surface ships with its money shot in the same
  landing, real renders only. Stageshow law governs demo
  material, and payload data never animates.
- Every round closes under the checkpoint rule: red team, the
  two-axis diff review, the seat's own re-verification, hosted
  gate green on the exact head, records and mirror in the same
  pass.
