# Dialect and price-table currency, Round 3 design record

Settled 2026-11-23 before the build, the same discipline as
Rounds 1 and 2. The roadmap charges this round with
re-deriving each dialect against the conventions as they
stand now, fixtures before support for new shapes, and a
refreshed price table with its date advanced and a changelog
line in DECISIONS.md. A research pass over the primary
sources ran first, and every change below rests on a source
fetched on 2026-11-23. Where the evidence ran out, this
record says so and builds nothing on the gap.

## What the research established

The GenAI conventions have moved house. The last released
text is v1.41, and the work now continues in a dedicated
repository with no release yet, so the adapter pins its
language to the released v1.41 text plus the named deltas
below. Nothing the adapter reads has been removed. The
Claude Code dialect's core fields are unchanged and its
cost field is intact, so that dialect needs no repair this
round. The Codex dialect is likewise unchanged in shape,
and Codex still exports tokens with no cost, so its gap is
price rows, not parsing.

## The GenAI cache write spelling

Cache creation has a new canonical attribute name,
gen_ai.usage.cache_write.input_tokens, and current emitters
use it. The adapter reads only the older
cache_creation spellings, so on a current emitter its
cache creation count silently reads zero. That is the one
silent data loss this round exists to fix. The adapter will
read the cache_write spelling first, then
gen_ai.usage.cache_creation.input_tokens, then the legacy
gen_ai.usage.cache_creation_tokens, each a fallback for the
emitter generation before it. Cache creation stays counted
and never priced, as it has always been. A fixture carrying
only the cache_write spelling lands first, asserting the
same figures the legacy fixture asserts for its own
spelling.

## New GenAI operations

invoke_workflow is a first-class operation in the current
text and the spec makes it the root of a run. The adapter
classifies it as agent structure, the same role invoke_agent
already carries. generate_content joins chat and its
siblings as a model call. plan stays a transparent
container, because the current text marks it internal and
conditional on planning being reliably distinguishable,
which a reader of spans cannot promise. One fixture stages
a workflow root over an agent over a chat call, a
generate_content call, and a tool call, and asserts the
structure and the turn totals.

## OpenInference reranker

The kind list has grown to twelve and RERANKER stands in it
as a first-class kind with its own attribute schema. The
adapter maps it to a tool step, the role RETRIEVER already
carries, named by tool.name or by the span name like every
OpenInference tool step. A fixture places a reranker step
between a retriever and a model call inside a chain and
asserts it survives into the run.

Two neighbouring kinds are decided against, with reasons.
DECISION stays a transparent container. The kind exists to
keep a decision model's tokens out of LLM token counts, and
the canonical run model has no decision step, so mapping it
to a model call would break the very separation the kind
declares, and mapping it to a tool call would mislabel a
model call in the waterfall. Its children still walk. The
decision reopens when the canonical model grows a decision
step of its own. Span-carried cost, the llm.cost attributes
some emitters now set, is not read. The renderer's own
table is the single pricing law in every dialect, its date
rides on every priced run, and a second pricing source
inside the spans would fork the honesty rules INV-38
states.

## The price table

The table advances its date to 2026-11-23. Every existing
row was rechecked against the official pricing pages and
none has moved. New rows land for the models current traces
actually name. On the OpenAI side, gpt-5.1-codex-max joins
its family at the same prices, the gpt-5.2 family and
gpt-5.3-codex land at their published prices, and the base
gpt-5, gpt-5-codex, gpt-5-mini, gpt-5-nano, and
codex-mini-latest rows land with them. On the Claude side,
which the shared table has never carried, so GenAI and
OpenInference traces naming Claude models land unpriced
today, rows land for claude-sonnet-5, claude-sonnet-5.5,
claude-opus-5.5, claude-opus-5 with the 4.5 through 4.8
Opus ids at the same published price, claude-sonnet-4.5,
claude-sonnet-4.6, claude-haiku-4.5, claude-fable-5.2, and
claude-mythos-5.2, each carrying its own published cached
price, because Anthropic's cache discount is no longer a
uniform ratio and a derived figure would be wrong. The
table's arithmetic is pinned row by row by unit tests in
the pricing module, so a future edit to any figure fails a
test naming the model.

Four price questions are recorded rather than resolved by
guessing. The gpt-5.1-mini row stays, because traces name
it and its figures stand corroborated by the identical
gpt-5-mini row, even though the current pricing page no
longer lists it under that name. The pro models carry no
published cached price, so no row lands for them. The
tiered claude-haiku-5.5 pricing does not fit the table's
flat row shape, so no row lands for it. GPT-6.2 models
entered the Codex catalog on 2026-11-20 with no published
API price, so Codex runs naming them land unpriced and the
run says so, which is the disclosure INV-38 already
requires. One further question is named for a later round.
Traces in the wild carry dated snapshot model ids, which
exact-match rows miss. Whether the table should normalise
a trailing date suffix is a pricing behaviour change with
its own failure modes, and it gets its own fixture-backed
design rather than riding along here.

## What this round deliberately does not build

Claude Code gained events the adapter does not read, among
them an api_refusal event, since a refusal arrives as a
successful stream and never fires api_error. Surfacing
refusals is worth doing, but the event's fields were not
verified from a live export in this pass, and this round's
discipline is fixtures before support, so it is named here
as the first candidate for the next convention pass and
nothing is built on it. The cache creation TTL split in
the same dialect adds nothing, because the adapter already
reads the summed field. No fixture is built on the
short-lived safety_event, which current Claude Code no
longer sends.

## Consequences for the corpus

Advancing the table date changes the pricing disclosure
printed on every priced agent run, so the priced goldens
regenerate and the gallery page is reassembled in the same
landing, the Round 2 freshness gate doing its work. No
existing fixture names a newly priced model, so no other
golden moves. The fences stand as they have stood in every
round. No datastore, no query language, no dashboard
editor, no alerting, no hosted renderer, no streaming
server, and no new product surface.
