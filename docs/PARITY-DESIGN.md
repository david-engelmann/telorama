# Round 9 design: parity completion

Status: design record for Round 9. It closes the
two asymmetries the Round 7 reviews named and
left standing. Round 7 made every surface rank
the same findings for the same session (INV-81).
What it did not do is make the incident scene
itself compose from the same inputs on every
surface, and it left the CLI's two input flags
able to double-count a span that arrives twice.

## The recon

Incident composition. The MCP incident tool
composes its scene over the full span set its
caller supplied. The CLI incident command takes
two inputs, the traces file and an optional
session, and composes over the traces file
alone, while ranking its rendered findings over
the union of both. One command therefore draws
two different span sets on one page: the scene
from the traces, the findings from the union.
Nothing documents the split as a decision; the
Round 7 red team found it by reading the two
call sites side by side.

Duplicate inputs. Both CLI commands build their
findings input by extending the trace spans with
the session spans, with no deduplication. A
caller who passes the same export under both
flags, an easy mistake when the session spans
came from the same trace export, counts every
span twice in the investigation. Counts inflate,
latency evidence doubles, and nothing in the
output says so.

## The finish

One combined input per command. Each CLI
command that takes traces and a session builds
its combined span set once: the trace spans
first, then the session spans, with each span
identified by its trace id and span id entering
the set once. The first occurrence is the copy
kept, the canonical-copy rule the sankey staging
has applied since Round 1. The combined set is
what the incident command composes from and
what both commands rank findings over.

Incident parity. The CLI incident command
composes its scene over the combined set, the
same rule the MCP tool applies to its single
input. The render command's scene data does not
change: the panels it was given still draw from
the trace spans alone, exactly as Round 7
settled, and only its findings input gains the
deduplication.

Pins. A CLI test passes one session whose spans
live only in the session input and asserts the
composed incident scene carries the panels
those spans produce. A second test passes the
same export under both flags and asserts the
findings equal the single-flag run's findings.
The existing parity test from Round 7 stands
unchanged.

## Fences

No change to the MCP tools, the scene spec, or
any renderer. No change to single-input calls:
a command given only traces produces
byte-identical output to today, because
deduplication removes nothing and no session
spans exist to compose over. The render
command's output does not change at all, since
its findings over disjoint inputs already ranked
the union and deduplication is a no-op there.
The one deliberate change is the incident
command with a session input, whose composition
now covers the spans its findings already
ranked. No new panel, dialect, or surface.

## Amendments, 2026-11-23

The reviews found one input class the fence
above states too broadly. The combined set
dedupes over the whole chained input, so a
single traces file carrying duplicate span ids
internally also loses its later copies, and its
output changes against the pre-round code. The
fence's byte-identity claim therefore holds for
duplicate-free inputs, which is the class every
fixture and golden exercises. INV-85 mandates
the behavior as built, each span id entering
once with no carve-out, and the deduped count
is the honest one for malformed telemetry.

The same exception qualifies the parity claim.
The MCP incident tool does not dedupe its
input, so on a duplicate-carrying export the
CLI and the MCP tool now compose different
counts where they once agreed on every input.
The agreement is exact on duplicate-free input,
and the CLI's side of the difference is the
single count the invariant requires. Widening
the dedup to the MCP tool is a change of its
own and is named here as a candidate, not
taken in this round.

One wording correction. The finish calls the
first-occurrence rule the canonical-copy rule
from Round 1; the sankey rule chooses its
canonical copy by the span's own fields,
independent of input order, while the combined
set chooses by position, traces before session.
The operative rule is the one stated in the
finish; only the lineage was loose.
