# Round 4 design record, the scale proof

Settled 2026-11-23 before the build, in the pattern the
earlier rounds set. The design record and the invariants
land first, the build follows against them.

## The charge

Round 4 of docs/ROADMAP.md. Ten-thousand-span trace,
five-hundred-turn session, hundred-thousand-sample
profile fixtures with hash-plus-headline assertions and a
CI timing budget. Past-linear degradations found by the
fixtures are fixed in the round and written into
INVARIANTS as bounds. The sankey design's known
sub-pixel-sliver limit at large volumes is owned here.

## What scale means for this engine

Every surface the engine ships is a pure function of its
input, so scale failure has only two shapes. The work
grows faster than the input, or the output grows faster
than the reader can use. The round tests both. Work is
measured by wall clock on the hosted runner against a
stated ceiling. Output is measured in bytes and in the
figures the reader sees, pinned by hash and by headline
counts so a silent change at scale fails the suite the
way a changed golden does at small scale.

## The three fixtures

The fixtures are generated inside the tests from fixed
arithmetic recipes, never committed as data files. A
ten-thousand-span JSON fixture would be megabytes of
unreviewable bytes, and the recipe is the reviewable
artefact. Each recipe is deterministic, so the hashes
below are stable across runs and machines.

The trace fixture builds ten thousand spans in one trace
with a shape chosen to stress the staging walks. A deep
chain of one thousand spans, wide fan-out beneath a few
hubs, eight services, and a tail of orphans entering
through the residual. Beside it a second fixture builds
the same span count as one thousand traces of ten spans,
because the sankey staging rebuilds its per-trace index
inside the trace loop and only a many-trace shape can
convict or clear that cost. Both render through the
waterfall, the service map, the sankey, and the flame.
Assertions are the FNV-1a hash of each rendered SVG, the
sankey conservation identities of INV-59 checked at
scale, and the span count the waterfall reports.

The session fixture builds a run of five hundred turns
directly, two model calls per turn, and builds the frame
stream whole. The run is constructed rather than adapted
from spans on purpose. The adaptation path is already
pinned at small scale by the ingest tests, and this
round's subject is the stream itself, whose input is the
run. The investigation runs over the run. Assertions are the
frame count, the final frame's headline totals, the hash
of the final frame's SVG, and the stream's total byte
count held against the ceiling INV-71 states.

The profile fixture builds one hundred thousand stack
samples across a fixed set of stacks and runs the flame
tree over them, and runs the heatmap grid over the trace
fixture's spans, the input that grid takes. Assertions are conservation,
the flame root carrying every sample and the heatmap
buckets summing to the sample count, and the hash of the
rendered flame SVG.

## The timing budget

Each scale test carries a wall-clock ceiling. The ceiling
is set at ten times the first green hosted run's measured
time and recorded beside the test, so the gate catches an
order-of-magnitude regression and never argues with
runner noise. A ceiling is a gross-regression gate, not a
benchmark. Nothing in the round publishes timings as
performance claims, and no ceiling is tightened without a
design note saying why.

## Suspects named in advance

Three costs are named before measurement, so the round
cannot quietly look away from them.

The sankey staging rebuilds its per-trace span index by
filtering the whole canonical map once per trace, which
is traces times spans when a payload carries many traces.
The many-trace fixture exists to measure exactly this.
If the evidence convicts it, the fix groups the canonical
spans by trace once, before the loop.

The frame stream's total output is quadratic in the turn
count by construction. Every frame is a complete scene
under the M5 law, so frame k carries k turns and the
stream sums to the square. That is not a defect to fix
but a bound to state and hold, which INV-71 does. The
per-frame work stays linear in the prefix, and the
five-hundred-turn fixture proves the absolute size stays
inside the recorded ceiling.

Prefix construction clones the run's turns once per
frame. The same fixture measures it inside the stream
total. If the clone dominates, the renderer learns to
draw a prefix without owning it, and the bound in INV-70
is re-stated with the new shape.

## The sankey sliver limit, owned

At ten thousand spans the sankey columns carry ribbons
whose heights fall below one pixel. The round's ruling
is that the diagram stays exact. Ribbons draw at their
computed heights, the figures printed on the panel stay
exact, and no top-N cutoff or other aggregation is
introduced, because dropping flow would break the
conservation law of INV-59 to make a prettier picture.
The limit is stated in INV-72 so a reader learns it from
the invariants rather than from a surprise.

## What this round does not do

No benchmark suite, no published timings, no criterion
harness, and no optimisation without a fixture that
convicts the cost first. No change to any scene, figure,
or byte at small scale, so the existing goldens and the
gallery stand untouched. No new panel, dialect, or
surface of any kind.

## Amendments, 2026-11-23

The build convicted two of the three suspects and one
defect the suspect list did not name. The many-trace
fixture measured 717 milliseconds against the single
trace fixture's 114 at the same span count, convicting
the staging walk's per-trace index rebuild. The fix
groups each trace's canonical spans once in the
canonical pass, as this record proposed. The prefix
cloning suspect was cleared. Building five hundred
frames measured 97 milliseconds, and the stream's
rendered total measured 104,428,401 bytes against the
ceiling INV-71 now records.

The unnamed defect was a panic. The service map's label
budget reached zero on the trace fixture's cyclic
service graph and the truncation helper subtracted
from zero, which fails outright in a debug build and
wraps silently in a release build. The truncation
helper is now total at a zero budget, the service map
floors its box width and label budget the way the
sankey narrows past seven stages, and INV-73 states
the rule. The layering itself still relaxes around a
cycle up to the node-count bound, so a cyclic graph
lays out deeper than its diameter. That is recorded
here as an observation. A cycle-aware layering is
design work for a later round, not a fix this round
makes.

## Amendments from the checkpoint reviews, 2026-11-23

The checkpoint reviews found the paper trail weaker
than the code, and the build was completed to match
this record. The thousand-trace fixture now renders
through a sankey over the whole set, so INV-69's
hash claim covers every fixture. The flame render
is proven on a wide trace of the same span count,
and the deep trace's flame refusal is pinned as a
named error. The renderer holds a standing bound
of 48 levels on flame trees, and the deep fixture's
span tree stands a thousand levels past it. The
bound is the product's own, set before this round,
and this round records it rather than relitigating
it. The per-node form
of the INV-59 identity is now checked at scale, not
only its aggregate consequences. The many-trace
ceiling was re-based on the fixed code's measured
82 milliseconds, since a ceiling set from the
convicted code's 717 could never catch that code
returning. Ceilings are rounded to the nearest
hundred milliseconds rather than whole seconds, and
the smallest figures carry the floor INV-70 now
states. The zero budget guard gained its own test
through the one caller that can still reach it, a
subagent nest twelve deep.

Two sentences in this record were loose and are
corrected here. The waterfall reports no span count.
Its output is pinned byte for byte by its hash, which
is the stronger check, and the count wording is
withdrawn. And the profile paragraph's heatmap
sentence mixed its units. The buckets sum with the
outside count to the trace fixture's span count,
while the sample count belongs to the flame tree
over the stacks.
