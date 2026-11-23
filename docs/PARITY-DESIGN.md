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
