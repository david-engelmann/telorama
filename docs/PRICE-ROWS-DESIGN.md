# Design record: price rows for the current model lines, Round 43

Settled 2026-11-25 before the build, the same discipline as
Round 3. Round 42 re-derived the price table against the
vendors' published prices and found no drift in any
existing row. It also found that the models published
since the table was compiled carry no rows, so a session
naming one lands unpriced and the run says so. This round
settles which of those models gain rows, what proves each
row before it lands, and what the addition may and may
not move. Every figure below was read firsthand from the
vendor's own page on 2026-11-25, and the page is named
beside its figures. Where the evidence ran out, this
record says so and builds nothing on the gap.

## The rows that land

Eleven model ids gain rows, joining the twenty-six the
table already prices. The new arms sit with their
provider's existing arms in the match in
crates/telorama-ingest/src/pricing.rs, grouped by shared
price the way the table already groups them. Rates are
US dollars per million tokens, stated as input, cached
input, and output.

| Model ids | Input | Cached input | Output |
| --- | --- | --- | --- |
| gpt-6.2-astra | 8.00 | 0.40 | 40.00 |
| gpt-6.2-sol and gpt-6.1-sol | 2.00 | 0.10 | 10.00 |
| gpt-6-astra | 10.00 | 1.00 | 50.00 |
| gpt-5.4 | 2.50 | 0.25 | 15.00 |
| gpt-5.4-mini | 0.75 | 0.075 | 4.50 |
| gpt-5.4-nano | 0.20 | 0.02 | 1.25 |
| claude-fable-5 and claude-mythos-5 | 10.00 | 1.00 | 50.00 |
| claude-fable-5.1 and claude-mythos-5.1 | 10.00 | 0.25 | 50.00 |

The four GPT-6 line figures stand on the OpenAI API
pricing page, whose flagship table lists exactly those
four ids at those rates. The GPT-5.4 figures stand on
the model pages OpenAI publishes for gpt-5.4, for
gpt-5.4-mini, and for gpt-5.4-nano, each printing its
own three rates. The Claude figures stand on the Claude
platform pricing page, whose model table prints the
input and output rates for all four ids and whose cache
hit table prints the read rate for the two lines, a
tenth of input for the Fable 5 and Mythos 5 line and a
fortieth of input for the 5.1 line. Each Claude row
carries its own published cached figure rather than a
derived one, the same discipline Round 3 recorded when
Anthropic's cache discount stopped being a uniform
ratio.

Two properties of the new ids need no new code. The
dated snapshot forms the model pages print, such as
gpt-5.4-2026-03-05, price by their base model through
the INV-84 transforms the moment the base row exists.
The Claude ids in their dash form, such as
claude-fable-5-1, reach the dotted row through the same
transform, the way claude-sonnet-4-5 already does.

## What stays unpriced, with reasons

gpt-5.4-pro gains no row. Its model page prints an
input and an output rate and no cached input rate, and
Round 3 already settled that pro models carry no
published cached price and so land no row. The build
pins that outcome with a test rather than leaving it to
the absence of an arm.

GPT-6.1 Terra and GPT-6 Luna gain no rows. The OpenAI
pricing page names both in a note on background request
rates and prints no token rates for either, and no
other vendor page fetched in this pass prints their
figures. They are recorded here as an open question for
the next currency pass, not priced from secondary
listings.

The domain configurations on the pricing page, the law
and healthcare variants and the trusted access models,
gain no rows. They are access-gated deployment
configurations rather than models the session dialects
name, and no fixture could honestly name one.

Claude Haiku 5.5 stays unpriced, as Round 3 settled.
The pricing page fetched for this record confirms its
input rate still steps at a hundred thousand tokens of
prompt, and a stepped rate does not fit the table's
flat row shape.

The long-context rates the GPT-5.4 and GPT-6 pages
print, doubling input and raising output past a stated
prompt length, gain no representation. The table has
never carried a context tier, the existing gpt-5.2 row
already lives under the same vendor rule, and adding a
tier dimension is a pricing behaviour change with its
own design. The batch, background, and fast rates stay
out for the same reason they have always stayed out.
Every row prices the standard tier.

## The Codex model id question

The GPT-6.2 rows exist chiefly for the Codex dialect,
whose exports report tokens and no cost. The vendor
pages give no codex-suffixed id for this line, the way
the GPT-5.1 generation carried gpt-5.1-codex and
gpt-5.1-codex-mini. The expectation is therefore that a
Codex export names the model it ran by its plain API
id, gpt-6.2-astra or gpt-6.2-sol. An expectation is not
evidence, and the standing rule is fixtures before
support, so the build round confirms the string before
either row lands. The Codex fixture below is staged
from a real Codex session export of a run on the
GPT-6.2 line, with token counts and timestamps adjusted
to the demo scale the other fixtures use and nothing
else rewritten. If the export names the model by some
other string, the fixture carries that string and the
row is keyed to it. If no real export can be staged,
the two GPT-6.2 rows and their fixture wait for a later
round and the rest of this round lands without them.

## Fixtures first

Each dialect that can carry the new ids gains a
fixture, a scene, and a golden, modeled on the
fixtures already committed. Every new scene joins the
gallery, so each build adds its section to the gallery
test and the page is reassembled in the same landing.

The Codex build adds
tests/fixtures/agents/codex-session-gpt62-logs.json,
modeled event for event on codex-session-logs.json. One
conversation runs two turns, the first naming
gpt-6.2-astra and the second gpt-6.2-sol, each turn
carrying its api_request and response.completed events
with input, output, and cached token counts, and one
tool decision and result between them. The scene is
tests/fixtures/scenes/agent_run_codex_gpt62_scene.json
on the shape of agent_run_codex_scene.json, and the
golden is tests/golden/agent_run_codex_gpt62_scene.svg.
An ingest test on the pattern of
codex_session_prices_tokens_and_pairs_tools asserts
both ids in the priced list, an empty unpriced list,
and each turn's cost at the new rows.

The GenAI build adds
tests/fixtures/agents/genai-session-current-spans.json,
modeled on genai-session-spans.json. An invoke_agent
root stands over chat spans naming gpt-5.4,
gpt-5.4-mini, gpt-5.4-nano, gpt-6-astra, and
gpt-6.1-sol, each span carrying its usage in the
attribute spellings the adapter already reads. The
scene is
tests/fixtures/scenes/agent_run_genai_current_scene.json
and the golden is
tests/golden/agent_run_genai_current_scene.svg. An
ingest test asserts all five ids priced, none unpriced,
and the run total equal to the sum at the new rows.

The OpenInference build adds
tests/fixtures/agents/openinference-session-fable-spans.json,
modeled on openinference-session-spans.json. An AGENT
span stands over LLM spans naming claude-fable-5,
claude-mythos-5, claude-fable-5.1, and
claude-mythos-5.1 in llm.model_name, with prompt,
completion, and cache read counts in the attributes
the adapter already reads. The scene is
tests/fixtures/scenes/agent_run_openinference_fable_scene.json
and the golden is
tests/golden/agent_run_openinference_fable_scene.svg.
An ingest test asserts the four ids priced and the
total at their rows.

The Claude Code dialect gains no fixture, deliberately.
Claude Code exports carry their own cost and the
adapter reads it, so the new Claude rows never price a
Claude Code run. The rows serve the span dialects,
where a Claude model arrives with tokens and no cost
of its own.

The pricing module's own tests carry the arithmetic
proof for every row, as they already do.
every_table_row_is_pinned gains one pin per new id,
eleven in all, each pinning its row's three figures at
a million tokens of a single class. The unpriced test
keeps its standing assertions, because the bare gpt-6.2
id and claude-haiku-5.5 still carry no row, and gains
one asserting gpt-5.4-pro stays unpriced.

## The as-of rule

PRICE_TABLE_AS_OF names the date the table's contents
were compiled. It advances when the table's contents
change, when a row lands, moves, or is removed, and it
takes the landing date of that change. A re-verification
that finds no drift does not advance it. That is the
reading Round 42 acted on when it left the date at
2026-11-23 and carried its re-verification in
DECISIONS.md instead. This round changes the contents,
so the build round sets the constant to its own landing
date, and every assertion and surface carrying the date
moves with it in the same landing.

## The no-repricing fence

Adding rows and advancing the date must not change any
existing figure. The priced surfaces stand as follows.

Four committed goldens print the disclosure line that
carries the date, the Codex scene, the Codex unpriced
scene, the GenAI scene, and the OpenInference scene.
No existing fixture names a newly priced model, so
every figure in those files stands, and each file moves
only in the date text of that one line. The unpriced
scene keeps naming gpt-unknown-9, which gains no row.
The Claude Code goldens print no table date and read
their cost from the export, so they do not move at all.
Every other golden carries no pricing and does not
move.

The gallery page at docs/gallery.html embeds the same
renders, so it is reassembled and moves only where its
embedded agent run renders carry the new date, apart
from the three new sections this round adds. The
viewer record at web/data/record.json is reassembled
from the same fixtures, so it moves only where a
session frame or render prints the date, with every
dollar figure unchanged.

The proof in the build round is the diff itself. The
regenerated files are reviewed file by file, and any
changed figure-bearing line in an existing surface
stops the landing. The pin test is the second half of
the proof, because it pins every existing row's three
figures by name, so an accidental edit to an old figure
fails a test that names the model.

## The invariant

INV-68 governs the table's date, its row-by-row
pinning, the naming of unpriced models, and the rule
that span-carried cost is never read. It does not state
the admission discipline this round and Round 3 both
worked under, that a row exists only behind a fixture
and a published price. The build round adds INV-93 in
the Dialect currency section of docs/INVARIANTS.md,
in this text.

INV-93. A model id carries a price row only when a
committed fixture names that id in a dialect export and
a price published by the model's vendor stands behind
every figure in the row. An id failing either condition
has no row, lands unpriced, and is named by the run
under INV-38.

The build round completes the invariant's enforcement
line with the tests it lands, on the pattern of the
neighbouring invariants.

## What the build round changes

- crates/telorama-ingest/src/pricing.rs gains the eight
  arms covering the eleven ids, grouped as the scope
  table groups them, and PRICE_TABLE_AS_OF advances to
  the landing date. Its unit tests gain the eleven pins
  and the gpt-5.4-pro unpriced assertion. No other
  production code changes, because every dialect already
  reads the model string and asks this one table.
- tests/fixtures/agents/codex-session-gpt62-logs.json,
  tests/fixtures/agents/genai-session-current-spans.json,
  and
  tests/fixtures/agents/openinference-session-fable-spans.json
  land as new fixtures, the Codex one staged from a real
  export under the gate stated above.
- tests/fixtures/scenes/agent_run_codex_gpt62_scene.json,
  tests/fixtures/scenes/agent_run_genai_current_scene.json,
  and
  tests/fixtures/scenes/agent_run_openinference_fable_scene.json
  land as new scenes.
- tests/golden/agent_run_codex_gpt62_scene.svg,
  tests/golden/agent_run_genai_current_scene.svg, and
  tests/golden/agent_run_openinference_fable_scene.svg
  are generated on the hosted generation route with
  UPDATE_GOLDENS=1. The four existing priced goldens
  regenerate in the same run and move only in their
  date line.
- crates/telorama-ingest/tests/ingest_tests.rs gains
  the three fixture tests, and its table_as_of assertion
  in codex_session_prices_tokens_and_pairs_tools
  advances to the new date.
- crates/telorama-svg/tests/golden_tests.rs gains the
  three golden tests on the patterns of their sibling
  scene tests.
- crates/telorama-svg/tests/gallery_tests.rs gains the
  three sections, the Codex pair joining the existing
  Codex loop and the other two standing beside their
  dialect blocks, and docs/gallery.html is reassembled
  with UPDATE_GALLERY=1.
- web/data/record.json is reassembled with
  UPDATE_RECORD=1 and moves only by the date where its
  sessions print it.
- crates/telorama/tests/cli_tests.rs and
  crates/telorama-mcp/tests/mcp_tests.rs each advance
  the one table_as_of expectation they assert.
- docs/INVARIANTS.md gains INV-93, and DECISIONS.md
  gains the build round's entry recording the rows, the
  new date, and the Codex id the export confirmed.

## What this round deliberately does not build

No adapter changes. No context tier, batch tier, or
speed tier enters the table. No row lands for a model
no fixture names, and no figure enters the record from
any source but the vendor's own page. The fences stand
as they have stood in every round. No datastore, no
query language, no dashboard editor, no alerting, no
hosted renderer, no streaming server, and no new
product surface.
