# Round 8 design: hostile inputs

Status: design record for Round 8. It takes up the
three named leftovers that share one shape: real
exports the current code handles badly. A cyclic
trace hangs one derivation and inflates another,
a refused model call vanishes from the session
record, and a dated snapshot model id silently
goes unpriced.

## The recon

Cycles. The waterfall's critical path walk in
derive.rs follows children by latest end with no
visited set, so a trace whose parent links cycle
below a root loops forever. The service map's
layering has the same input class: a cyclic
service graph inflates the layer count instead
of collapsing the cycle. Round 4 found both and
named them as observations outside that round's
diff. A hang is the worst defect class this
codebase can ship, because every surface that
draws a waterfall inherits it.

Refusals. Claude Code emits a
claude_code.api_refusal event when the model
refuses a request. Round 3 verified the event
exists and that nothing reads it, so a refused
call leaves no trace in the run: no step, no
error, no token count. The session reads as if
the call never happened, which breaks the
codebase's standing rule that the record states
what the sources said.

Snapshot ids. The price table matches model ids
exactly. Real exports usually carry the dated
snapshot form, claude-sonnet-4-5-20250929 or
gpt-5.1-2025-11-13, and the dash version form,
neither of which matches a table row, so the
common case prices nothing and the run reports
itself unpriced. Round 3 deferred the question
by name.

## The finish

Cycle safety. Every derivation terminates on
cyclic input. The critical path stops at the
first repeated span, so the path is the longest
walk that visits no span twice. Service map
layers come from the condensation of the graph:
spans in one strongly connected component share
a layer, and layers are the longest path over
the component graph. On acyclic input both
derivations produce exactly today's output,
which the existing goldens and scale hashes pin;
on cyclic input they terminate with the cycle
collapsed. A cyclic fixture at unit scale and
one at the scale suite's width pin the behavior.

Refusals are visible. The Claude Code adapter
reads claude_code.api_refusal events, fixtures
first, and each refusal becomes a model call in
its turn with the error flag set and zero
tokens, carrying the model the event names. The
turn's step list and the investigation's errors
ranking then see the refusal like any other
failed call. A fixture session with one refusal
pins the shape end to end, from the parsed run
to the rendered strip.

Snapshot ids price by their base model. Pricing
lookup tries the exact id first. It then strips
one trailing date suffix, in either the
YYYYMMDD or the YYYY-MM-DD shape, and tries
again. It then reads the dash version form as
the table's dot form and tries once more. Every
step ends in an exact match against a table
row, so a transform can only price a model the
table carries under its canonical name, and an
id that matches nothing stays unpriced and is
reported as such. Tests pin the dated and dashed
forms of priced models in both providers'
shapes, and pin that an unknown base model
stays unpriced after every transform.

## Fences

No output changes on acyclic, refusal-free,
canonically named input: the golden corpus and
the scale hashes stand untouched. No fuzzy or
prefix pricing matches at any step. No new
panel, dialect, or surface. The refusal work
follows the standing fixtures-before-support
discipline from Round 3.

## Amendments, 2026-11-23

The build found a fourth walk the recon missed:
flame_from_spans built its tree by unbounded
recursion, so a cyclic trace overflowed the
stack instead of hanging. It now builds each
span id once, under the same rule as the
waterfall rows. The recon's two named sites and
the layering are fixed as designed; the rows
walk, named in INV-82's every-derivation
letter, carries the same guard.

The refusal paragraph above first said the
investigation's errors ranking would see the
refusal. That was wrong about the existing
behavior: the errors ranking is span-based, and
failed model calls, api_error calls included,
surface in the turn's step list and the error
drawing, not in the ranking. The build gives a
refusal exactly the treatment an api_error call
already gets, and INV-83 states that treatment.
