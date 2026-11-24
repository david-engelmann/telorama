# Design record: the verdict viewer for telorama (Round 20)

## Why this round exists

David's frontend ruling set the standard of the verdict-viewer
treatment for the fleet's showcase surfaces, crosscheck first and
telorama next. The gallery already claims that standard as a look.
This record defines the treatment itself for telorama, as a surface
of its own, before any build. Crosscheck's viewer stands finished in
its repo and this record transfers its pattern, not its domain.

## What the treatment is, from the crosscheck original

Crosscheck's viewer renders one recorded, real run of its tool over
one real report. The source document stays center stage with the
machine-checkable claims highlighted in place at their exact
anchors. A summary bar carries the counts per verdict class and the
run's pass or fail consequence. Clicking a highlighted claim or a
row in the claims list opens an inspector that shows the claim, its
evidence, what the check found, and a plain statement of why the
verdict came out as it did. The footer prints the tool's limit line,
what the run does not prove, and discloses the corpus's planted
cases. Nothing is verified at view time. The page is a static
export, a pure function of two committed files, the run record and
the source artifact, and the record is produced by a script that
runs the real checks rather than writing results by hand. The demo
corpus is chosen so every verdict class the tool can produce is
visibly exercised.

## The telorama mapping

The source artifact is a rendered scene, and for a session record a
frame stream. The claims are the investigation layer's findings,
which already attach to every render with no suppressing flag
(INV-57) and already carry their evidence and exemplars as
structured data. The verdict classes are the finding severities.
The anchors are the panels and spans a finding names, marked in the
delivered SVG the way crosscheck marks character spans in prose.
The inspector shows the finding's statement, the figures it rests
on, its exemplars, and why it ranked where it did, in the finding
layer's own terms. The summary bar carries the finding counts per
severity and the investigation's own headline, the look-here-first
line the tools already print. The limit line states what a render
record does not prove, that the findings are derivations over the
recorded data and nothing was recomputed in the page.

## The run record

One record, committed under web/data, produced by a generator that
drives the real tool paths over committed fixtures and writes what
they return. The record carries the scene spec, the delivered SVG
for the scene and for every frame where the record is a session,
the findings exactly as the render path computed them, the derived
summary figures, the engine version, and the content hash of each
SVG, the same hash the CLI frame stream already emits per frame.
The record's corpus is the incident composition and one session,
chosen so the findings cover every severity the investigation layer
emits and at least one panel carries no findings, so the page shows
the quiet case as well as the loud ones. A freshness gate in the
gallery's pattern regenerates the record and compares it byte for
byte, so the page cannot drift from the product it shows.

## The page

A web directory beside the crates, Next.js and React with a static
export and one hand-written stylesheet, the crosscheck viewer's
stack and shape. A masthead with the name, one plain sentence, and
the record's provenance. The summary bar. The scene rendered large
from the delivered SVG, with a frame stepper where the record
carries frames, stepping replacing the whole frame exactly as the
MCP Apps view does. The findings list beside the scene, each row
selectable, the selected finding marked in the scene and opened in
the inspector. The footer with the limit line and the corpus
disclosure. The page fetches nothing and computes nothing. It
displays delivered artifacts only.

## Fences and the new invariant

The gallery is untouched and INV-64 stands as written. The
standalone HTML export is untouched and INV-39 stands. The viewer
is showcase material under the Stageshow law, stated in its own
readme and footer the way crosscheck states it. Payload data never
animates, no figure transitions, no intermediate frames are
synthesized, and stepping replaces whole frames, the INV-56 pattern
carried to the new surface. No client-side derivation exists to
diverge from the renderer, because the page receives finished SVG
and finished findings, so INV-18 is never at risk. The surface adds
no datastore, no query language, no editor, and no server, and it
renders nothing live, so it is not a hosted renderer. It is a
recorded run in a page, the same category the gallery occupies,
opened as its own surface because the gallery's no-script rule
forbids growing it there.

One invariant joins the set when the build lands. INV-91, the
viewer renders one recorded run from committed data and computes
nothing at view time, displaying only the delivered SVG and the
recorded findings, with whole-frame replacement on stepping and no
animation of payload data, enforced by the record's freshness gate
and the viewer surface tests.

## Scope fences for the build

No changes to any crate's rendering, derivation, or tool surface.
No changes to the gallery, the standalone export, or the MCP Apps
view. The generator uses the tools as they stand. The build lands
the web app, the committed record, the generator, the freshness
gate, and INV-91. Hosting the static export is out of scope, as it
is for crosscheck. A money-shot video of the viewer belongs to the
showcase pass after the build, under the Stageshow standard.

## Amendment from the build (2026-11-23)

Two phrases in this record met the code during the build and were
settled as follows. The investigation layer has no severity. Its
classes are the four finding kinds, errors, latency, wait, and
cost, ordered by rank, and the page's summary counts and colors
key on kinds. The record also carries one derivation the text
above did not name. Findings carry no panel field and the
delivered SVG carries no anchors, so the generator derives each
finding's panel from its kind and the scene's panel list, and the
record carries the anchors as data. The page marks a finding by
that panel reference in the inspector and the list. It does not
highlight inside the SVG, because the SVG has no anchors to
highlight and this round's fence forbids renderer changes. Under
that derivation the incident heatmap is anchored by no finding,
which is the quiet panel the corpus section requires, and the
freshness gate asserts it.
The build's red team adds one precision to the anchor story. The
derivation is kind-level and subject-blind, and it is honest on
this corpus because the composition is fixed. The latency
finding's exemplar sits in the exemplar trace the waterfall
draws, which is what makes its anchor true. A fixture whose
slowest span sat outside the exemplar trace would keep the gate
green while the anchor stopped telling the truth, and that is
the point where the derivation would need the finding's
exemplars as well as its kind. The heatmap's quietness is an
anchor fact. The panel still draws every span in the window,
bucketed by duration, with the findings' spans anonymized inside
the aggregate.
Round 21 closed that exposure. The derivation now reads the
finding as well as its kind. The latency anchor requires the
waterfall's data entry to carry the finding's exemplar trace,
and the errors anchor requires the error rate row to be
unfiltered or filtered to the finding's service, with synthetic
divergence pins on both. A panel that draws something else earns
the finding no anchor rather than a false one.
