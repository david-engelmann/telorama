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

## After the roadmap: Round 7, the documented surface

The six rounds above are complete. Round 7 takes up
the two items the registry round left named: the
public surface documented field by field under a
permanent missing-docs gate, and findings parity
between the CLI and the other surfaces. Design
record docs/DOCS-DESIGN.md. Named candidates
beyond it: Claude Code refusal events,
dated-snapshot model ids, and cycle-aware
service-map layering.

## Round 8: hostile inputs

Round 8 takes up the three candidates named
above, which share one shape: real exports the
code handles badly. A cyclic trace hangs the
waterfall's critical path and inflates the
service map's layers, a refused model call
vanishes from the session record, and a dated
snapshot model id silently goes unpriced. Design
record docs/HOSTILE-DESIGN.md.

## Round 9: parity completion

Round 9 closes the two asymmetries the Round 7
reviews named and left standing: the CLI
incident command composes over its traces alone
while the MCP tool composes over its full
input, and the CLI's two input flags can count
one span twice. Design record
docs/PARITY-DESIGN.md.

## Round 10: the named leftovers

Round 10 takes the three leftovers Rounds 8
and 9 recorded without fixing: the MCP tools
do not dedupe their input where the CLI now
does, service_layers can panic on a hand-built
graph whose edge names an absent service, and
the flame builder's recursion can overflow on
a very deep acyclic trace. Design record
docs/LEFTOVERS-DESIGN.md.

## Round 11: depth, finished

Round 11 takes the two depth paths Round 10
named: the folded-stack builder still inserts
and freezes recursively, and the MCP panel
summary and detail paths still drop the flame
trees they own. One shared dismantle in
telorama-scene, an iterative folded-stack
builder, and the INV-86 exception clause
retires. Design record docs/DEPTH-DESIGN.md.

## Round 12: the agent tree

Round 12 takes the two agent-tree paths Round
11's sweep named: the dialect session
builder's emit recurses over an acyclic span
tree, and the agent-run layout walk recurses
over a run's nesting, with the run trees' own
drops on the same paths. Both walks iterate,
and one nesting budget of 512, enforced where
nesting is built, bounds every run tree.
Design record docs/AGENT-TREE-DESIGN.md.

## Round 13: the registry, refreshed

The crates stand at 0.2.0 while six rounds of
documenting and hardening have landed since.
Round 13 versions all five crates together at
0.3.0 and publishes them in the settled order,
with the readmes' state passages rewritten for
the finished surface and no behavior change.
Design record docs/REGISTRY-REFRESH-DESIGN.md.

## Round 14: the long session

Round 4 proved the frame stream at five
hundred turns. Round 14 proves the session
side at five thousand turns and ten thousand
events, under the same generated-fixture and
ten-times-ceiling rules, and states the
practical session ceiling in the record if
the frame stream's quadratic cost convicts
itself. Design record
docs/LONG-SESSION-DESIGN.md.

## Round 15: the second sweep

Rounds 8 through 12 hardened the derivations
walk by walk. Round 15 aims the same
adversarial sweep at the boundaries no round
has audited as a set, the text, parse,
numeric, remaining-walk, and protocol
boundaries, and fixes what the audit
convicts. Design record
docs/SWEEP-DESIGN.md.

## After Round 15: the programme since

The programme did not stop where this
roadmap's sections stop. Rounds 16 to 18
hardened identity in the derivations.
Duplicate span ids are attributed by
occurrence under INV-89, on the trace side
in Round 16 and on the session side in
Round 17, and permission decisions pair to
their results by occurrence under INV-90
in Round 18. Round 19 released all five
crates at version 0.4.0 on those rounds.

Rounds 20 to 25 built the verdict viewer
and completed its recorded run, the viewer
itself, its anchor truth hardening, the
scene reel, the Codex session, all four
dialects in the record, and the reel
refreshed for the completed record. Round
26 re-checked the MCP Events watch and
left it standing on its two named
triggers. Rounds 27 and 28 ran the
accuracy programme over the README and
the rest of the documented surface,
correcting only what the audits proved
stale.

Rounds 29 and 30 gave the OpenInference
dialect its own golden scene. Rounds 31
to 34 made text width honest, a layout
width read from character classes under
INV-92 in place of a raw character count,
with the width tables then derived from
the Unicode Character Database rather
than drawn by hand. Round 35 released all
five crates at version 0.5.0 on that work.
Rounds 36 and 37 re-proved the Round 14
scale figures under the width model, and
Round 38 pinned the wide names scene in
the dark theme as well as the light.
Rounds 39 and 40 renewed the accuracy
passes over the README and the documented
surface.

Rounds 41 to 45 kept the record and the
price table current. Round 41 brought this
roadmap current, adding the closing section
that records the programme from Round 16.
Round 42 re-checked the dialect and price
currency and found no drift, the table's
date staying at 2026-11-23 because a
re-verification alone does not advance it,
and the GenAI pin holding against the
conventions as they now stand. Rounds 43
and 44 designed and then built price rows
for the current model lines under INV-93,
which admits a row only behind a committed
fixture and a price its vendor published.
Nine of the eleven designed ids landed and
the table's date advanced to 2026-11-25 as
its contents changed, while the GPT-6.2
pair waits on the model string a real
Codex export of a GPT-6.2 run carries, its
ids pinned rowless until one can be
staged. Round 45 ran the README accuracy
pass for the third time, thirty-five
claims audited and the two the intervening
rounds had staled corrected.

Rounds 46 to 49 swept the documented
surface again, shipped the next release,
refreshed the reel, and re-checked the
MCP ground. Round 46 ran the docs surface
accuracy sweep for the second time, forty
claims audited and thirty-seven standing
true, and its three corrections landed,
this roadmap carried through Round 45,
the wording of INV-48 matched to its own
enforcement, and the dossier given a
closing note naming the five crates as
built. Round 47 released all five crates
at version 0.6.0, and the publish record
stands with the crates live on the index
and their documentation served by docs.rs.
Round 48 recomposed the viewer scene reel
from the record as it now stands, its
frames carrying the price table date of
2026-11-25 and the engine stamp of the
new version. Round 49 read the MCP facts
again firsthand and found MCP Apps and
the Events extension unchanged, SEP-1865
still Final and Events still in Beta, so
the watch continues on its two named
triggers. The same round recorded a gap
it did not close, the core protocol now
standing at revision 2026-07-28 against
the 2025-06-18 telorama negotiates, with
the older revision final and frozen and
nothing in the move breaking it.

Rounds 50 to 55 brought the record
current again and widened the dark
corpus. Round 50 carried the README
count to forty-nine rounds and this
roadmap's closing section through
Round 49. Round 51 settled the protocol
revision move in a design record,
dual-era service from the one stateless
handler, its build waiting on the
owner's lock. Rounds 52 and 53 audited
the invariant traceability and repaired
what it found, one cited test missing
and its behaviour untested, so the test
now stands under the cited name and
every enforcement line names tests that
resolve in the tree. Rounds 54 and 55
pinned the OpenInference fable scene
and the GenAI current scene in the dark
theme beside the light, eight of the
seventeen scenes now pinned in both
themes and the golden corpus standing
at twenty-five.

Rounds 56 to 66 swept the documented
surface a third time, refreshed the
rendering-host snapshot, and carried the
dark corpus to completion. Round 56 ran
the docs surface accuracy sweep for the
third time, seventy-nine claims audited
and seventy-seven standing true, its one
correction carrying this closing section
through Round 55, and one claim left
unverified by design, the dossier's
rendering-host snapshot, dated research
framing awaiting a research pass of its
own. Round 57 was that pass, the
snapshot re-derived firsthand from the
vendors' current documentation, every
host the November snapshot named standing
where it stood, Microsoft 365 Copilot
joining the rendering set, and Goose's
entry gaining the desktop qualifier its
own documentation supports, with two
findings recorded rather than acted on,
the ui.domain guidance joining the watch
list and the new host rendering only
from remote servers a local stdio server
does not reach. Rounds 58 to 66 then
pinned the nine scenes the dark corpus
still lacked, the API, heatmap, and
service map scenes, the Codex, GenAI,
and OpenInference agent run scenes with
the unpriced Codex run beside them, the
incident checkout scene, and the agent
state scene last, each dark golden
generated on the hosted route from its
scene as it stood and each README count
moving with its pin. Every one of the
seventeen scenes now stands pinned in
both themes, and the golden corpus
stands at thirty-four, seventeen light
and seventeen dark.

Rounds 67 to 71 renewed the record,
closed the viewer's owed verification,
and made the reel a recording of the
page itself. Round 67 ran the README
and roadmap currency audit again and
carried both files current through
Round 66. Round 68 audited the
invariant traceability a second time
and found it clean, every enforcement
line naming tests that resolve in the
tree. Round 69 verified the viewer
export on the hosted route against the
committed record, the page building
clean from the record as it stands, and
recorded that the web project carried
no committed lockfile. Round 70
committed that lockfile, proven by a
clean install and build on the hosted
route before the pull request opened.
Round 71 discharged the owed page
capture on the same route, the reel now
a recording of the viewer page stepped
through its own controls in place of
the composition of rendered frames that
stood before it.

Rounds 72 to 78 renewed the accuracy
programme across every surface and
shipped the next release. Round 72 ran
the README and roadmap currency audit
for the fourth time, eighty claims
audited and seventy-seven standing
true. Round 73 swept the documented
surface for the fourth time,
eighty-eight claims audited and
eighty-six standing true, its two
corrections carrying the discharge of
the viewer's owed verifications into
the design records that had left them
standing as owed. Round 74 audited the
examples and plugin surface for the
first time, one hundred and eighteen
claims audited and one hundred and
sixteen standing true. Round 75
audited the viewer page copy for the
first time and found all thirty-nine
strings true. Round 76 audited the CLI
and MCP surface text for the first
time, fifty-nine claims audited and
fifty-five standing true, its
corrections stating the input choice
on the session and frames usage lines
and narrowing the incident tool's
schema to the arguments it reads.
Round 77 audited the crates.io and
docs.rs surface for the first time and
found all sixty-seven claims true.
Round 78 released all five crates at
version 0.7.0 on that work, the viewer
record regenerated so its engine stamp
reads the new version at an unchanged
byte count.

Rounds 79 to 85 renewed the record,
audited the surfaces for completeness,
re-checked the currency, and proved the
gallery fresh. Round 79 ran this audit
for the fifth time, one hundred claims
across the two files and ninety-eight
standing true. Round 80 audited the
invariant traceability a third time and
found it clean, every cited test name
resolving in the tree. Round 81 asked
whether the README says everything the
CLI surface offers and added the schema
command and the two render flags it had
left unnamed. Round 82 asked the same
of the plugin skill and gave the sankey
and agent state panels their paragraphs
there. Round 83 re-checked the dialect
and price currency and found no figure
drifted, the table date staying at
2026-11-25, while the pin header in the
GenAI adapter was corrected beside the
record, the conventions repository
having cut its first release two days
before the pin. Round 84 asked whether
the examples walk through everything
the product teaches and closed the four
gaps it found, the schema command, the
flame and sankey panels, and the agent
state panel now worked end to end.
Round 85 regenerated the gallery on
the hosted route and found the
committed page byte-identical to the
fresh generation.

Rounds 86 to 98 renewed the record
again and re-ran every accuracy pass
the programme keeps. Round 86 ran this
audit for the sixth time, one hundred
and sixty-nine claims across the two
files and one hundred and sixty-seven
standing true. Round 87 checked the
viewer reel's currency for the first
time and found it stale in one
rendered line, the engine stamp the
Round 78 release had moved. Round 88
re-captured the reel on the hosted
route, H.264 at 1280 by 800 and 41.08
seconds, 890,153 bytes, and restated
the standing rule that a release
regenerating the record owes the reel
a re-capture when the stamp moves.
Round 89 swept the documented surface
for the fifth time, one hundred and
forty-one claims and two staled, both
carrying the GenAI conventions
no-release premise, each corrected
beside the sentence that carried it.
Round 90 re-checked the MCP currency
a second time and found nothing
moved. Round 91 audited the invariant
traceability a fourth time and found
it clean, its figures identical to
the third running in every dimension.
Round 92 audited the examples and
plugin surface a second time, one
hundred and twenty-four claims and
none staled. Round 93 audited the
viewer page copy a second time and
found all fifty-six strings true.
Round 94 audited the CLI and MCP
surface text a second time, seventy-
four claims and all true, the four
fixes of the first running verified
as landed. Round 95 audited the
crates.io and docs.rs surface a
second time and found all sixty-seven
claims true against version 0.7.0.
Round 96 audited the showcase media
for the first time and found all four
artifacts current. Round 97 swept the
documented surface a sixth time, one
hundred and forty-six claims and
every one true. Round 98 re-checked
the dialect and price currency a
third time and found no figure
drifted, the GPT-6.2 pair now priced
on its vendor's page yet still
rowless on the fixture half of
INV-93, and new deprecation colour
touching priced ids recorded beside
the figures it leaves unmoved.

Rounds 99 and 100 renewed this audit
and closed the programme. Round 99
ran the README and roadmap currency
audit for the seventh time, one
hundred and seventy-seven claims
across the two files and one hundred
and seventy-five standing true, its
two corrections carrying the README
count to ninety-eight rounds and
this closing section through Round
98. Round 100 ran the closing
full-state audit and reconciled the
programme end to end, the round
count exact from the record's own
headings, the tree at one hundred
and eighty-seven files, the golden
corpus complete at thirty-four with
seventeen scenes pinned in both
themes, the invariants file ending
at INV-93, all five crates published
at version 0.7.0, and the checks on
the main head green, with the five
open items named as carried forward,
each gated on something outside the
repository rather than on unfinished
work inside it.

DECISIONS.md carries the record of every
round, this one included, and stands as
the live account of the programme.

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
