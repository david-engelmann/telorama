# Round 15 design, the second sweep

## The gap

Rounds 8, 10, 11, and 12 hardened the
derivations. Every walk over span trees and
agent trees now terminates, iterates, and
respects the nesting budget, and the record
proves each one. The hardening followed the
findings, though, and the findings came from
sweeps aimed at the walks the previous round
had just touched. The surfaces no sweep has
aimed at are the boundaries, where caller
controlled input enters and where figures
leave the range the fixtures exercise.

Five boundary groups stand unaudited as a set.
Text boundaries, where scene titles, service
names, model names, and questions enter the
SVG, the HTML export, the MCP view, and the
text summaries. Parse boundaries, where scene
JSON, OTLP exports, and log documents arrive
at shape extremes. Numeric boundaries, where
timestamps, durations, token counts, and
figures reach the edges of their types.
The remaining walks, sankey staging under
combined duplicate and cyclic input, exemplar
selection, and the investigation layer over
adversarial runs. And the protocol boundary,
where the MCP server takes JSON-RPC it did
not write.

## The finish

Three audit lanes, each fenced read-only,
each owning one part of the charter below.
Findings are graded and evidenced. The build
then fixes what the audit convicts, each fix
with its pin, and records what the audit
verified without convicting. A surface the
audit clears is named as cleared in the
record, so the sweep's coverage is itself
part of the evidence.

The charter is this.

- Every place caller text enters markup is
  escaped for that markup, or the audit shows
  the text cannot reach it.
- Every parser refuses malformed or extreme
  input with a named error. Deeply nested
  documents, duplicate keys, invalid encodings,
  and missing fields are the audit's inputs.
- Arithmetic over timestamps, durations, and
  figures is checked at the edges of its
  types. Overflow, negative spans of time,
  and empty inputs produce a named error or a
  defined rendering, never a panic or a hang.
- The remaining walks terminate and stay
  honest under the combined hostile inputs
  the earlier rounds tested one at a time.
- The MCP message handler answers malformed
  requests with an error object. No request
  shape panics it.

## The invariant

One invariant joins the set when the build
makes it true.

- INV-88. Every boundary the workspace
  exposes refuses malformed or extreme input
  with a named error or a defined rendering,
  and caller text enters markup only escaped
  for that markup. No boundary panics, hangs,
  or emits unescaped caller text.
  Enforced by the sweep tests in the scene,
  ingest, svg, mcp, and cli suites.

## Fences

No behavior change on well-formed input. The
golden corpus and the scale hashes stand
unless a conviction forces a change, and a
forced change is named in the record with
its reason. No new panel, dialect, or
surface. The audit lanes edit nothing, and
the build takes only what the audit
convicts.

## Standing lanes

The A2UI watch is reviewed for this round
and stays a watch item. The convention check
rests on the charges pass of 2026-11-23, the
previous reading, unless the audit's protocol
work surfaces a divergence.
