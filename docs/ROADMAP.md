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
