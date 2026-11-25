# Design record: all four dialects in the viewer record

Round 24 completes the verdict viewer's recorded run. The record
carries two of the four session dialects the engine ingests, and
the sessions list Round 23 settled was shaped to admit the rest
without another rename. This record settles the extension before
the build. It is a small record because the shape work is done.

## The record

Two entries join the sessions list, in the order the CLI names
the dialects. The GenAI entry is assembled from
tests/fixtures/agents/genai-session-spans.json through
parse_genai_session over the parsed spans, and the OpenInference
entry from tests/fixtures/agents/openinference-session-spans.json
through parse_openinference_session. Both dialects are spans-only
in the CLI, requiring the spans input and never reading logs, so
each assembly is a single fixture through a single adapter call,
the same mold as the first two entries. The reranker variant of
the OpenInference fixture stays out, as the other variant
fixtures do. The incident half and the two existing entries do
not change.

Each new session carries one turn in its fixture, so each adds
one frame. The coverage test pins each new entry's session id,
dialect, frame count, and finding kinds exactly. The kinds are a
measurement the build confirms from the regenerated record
before pinning, as Round 23 did for Codex. The summary bar's
totals follow the record and need no page change, because the
page already counts the final frame of every session it is
given.

## The page

No page code changes. The sections already map over the sessions
in record order, each with its own stepper, and the summary loop
and its scope label already speak of each session. A one-frame
session renders a stepper with both directions disabled, which
is the honest rendering of a one-turn record.

One decision is recorded so it is not mistaken for an oversight.
The section copy prints the record's dialect string verbatim, so
the new sections read as recorded genai and recorded
openinference session exports, in the same form as the existing
claude_code entry. There is no display-name map on the page, and
this round does not add one. The record is the vocabulary.

## The invariant

INV-91 needs no amendment. Round 23 already worded it as one
recorded session per dialect the record carries, and this round
is that wording becoming fully true.

## Fences

No renderer, derivation, ingest, or page changes. No new
fixtures. The record re-pins once through the hosted generation
workflow in the established mold. The owed capture of the viewer
page and the owed local export verification are untouched by
this round. Both have since been discharged, the export
verification in Round 69 and the page capture in Round 71,
each on the hosted route.

## Amendment from the build (2026-11-24)

The measurements the record reserved are taken. The GenAI
session carries one frame whose only finding kind is cost, a
turn at $0.0231 on gpt-5.1, and the OpenInference session
carries one frame whose only finding kind is cost, a turn at
$0.0094 on the same model. Neither span dialect merges wait
data, so no wait finding arises from either, and the coverage
pins say exactly that with the reason stated, in the Codex
form. The summary bar's true totals on the completed record
are one errors, one latency, one wait, and four cost. The
incident half and the two existing sessions regenerated
byte-identical, as the fences required.
