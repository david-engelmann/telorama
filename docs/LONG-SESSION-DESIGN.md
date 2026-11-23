# Round 14 design, the long session

Status. Design draft, prepared 2026-11-23.
Lands as docs/LONG-SESSION-DESIGN.md in the
design PR.

## The gap

Round 4 proved the renderers at ten thousand
spans and a hundred thousand samples, and the
frame stream at five hundred turns, where it
measured 104,428,401 bytes against a ceiling of
105,000,000. Nothing proves the session side
past that point. Real agent sessions run far
longer than five hundred turns. The frame
builder clones the run prefix for every frame,
so the stream's cost grows with the square of
the turn count by construction, and no fixture
says where that stops being practical, or what
the honest ceiling is. The ingest side has the
same shape of gap. A session log with tens of
thousands of events has never been through the
adapters in a test.

## The finish

Generated fixtures in the Round 4 style, fixed
arithmetic recipes inside the tests, never
committed as data. A five thousand turn session
through the frame stream, asserting the frame
count, the final frame's headline figures, and
a byte ceiling set at ten times the measured
size on the first green hosted run, the Round 4
rule. A ten thousand event Claude Code log and
a ten thousand span GenAI session through the
adapters, asserting turn counts and totals.
Wall-clock ceilings on the same ten times rule.
If a fixture convicts a real defect, the round
fixes it and says so. If the frame stream's
quadratic cost convicts itself at five thousand
turns, the round states the practical session
ceiling in the record and the CLI names it when
a session passes it, rather than silently
slowing.

## Fences

No output change on any session the current
code handles. No benchmark suite, no published
timings, no optimisation without a convicting
fixture, the Round 4 fences unchanged.

## Standing lanes

The A2UI watch is reviewed for this round
against the spec's movement and stays a watch
item. The convention check reads the GenAI
conventions against the pinned commit and the
MCP Apps spec against the stable text, the
charges pass of 2026-11-23 being the previous
reading.
