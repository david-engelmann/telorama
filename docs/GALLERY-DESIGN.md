# The gallery, Round 2 design record

Settled 2026-11-22 before the build, the same discipline as
Round 1. The roadmap charges this round with a static gallery
page in the repo showing every scene family rendered from the
golden fixtures, each with its findings strip and its
provenance line, the incident scene and the frame stream at
its head, linked from the README's first screen. David's
frontend ruling adds the standard of the verdict-viewer
treatment, a page that sells the work on sight. Stageshow
law bounds the material to demo and showcase surfaces only,
real renders only, and no animation of payload data.

## What the page is

One static file at docs/gallery.html. No JavaScript anywhere
in it. A dark header band carries the name and one plain
paragraph about what telorama does. The head of the page
shows the two showcase pieces that already stand as landed
media, the frame stream stepping through a live session and
the incident still, each referenced from docs/media where
those files already live. Below the head, one section per
scene family in the golden corpus, twelve sections grouped
under family headings. The incident pair of goldens is one
family shown once, with its findings strip drawn in. Each
section prints the scene's title, the provenance question
the scene was generated to answer, and the rendered scene
itself.

## Where the pictures come from

Every picture is the renderer's own output through the tool
render path, the same path the command line tool uses. That
path computes the investigation layer's ranked findings over
the scene's data and draws the findings strip into the SVG
when findings exist, which is what the roadmap's findings
strip phrase means on this page. The eleven fixture scenes
render from their fixtures. The two incident scenes render
from the composed scenes the golden tests build from the
checkout traces. Nothing is drawn by hand and nothing is a
screenshot. The frame stream and the incident still at the
head are the landed media files, themselves renderer output
from the media pass.

## How the page is assembled and kept honest

The freshness test doubles as the assembler, the same
pattern as the golden corpus and its update flag. Run under
its update flag on a CI runner, the way the README media was
assembled, it writes the page from the shell at
docs/gallery-shell.html and the renders it computes. Run
normally, it asserts the committed page is that assembly,
byte for byte. This machine cannot be trusted with the
write, so assembly runs on the runner and the page lands as
a committed file. Any change to a fixture, a finding, or the
renderer fails the freshness test until the gallery is
assembled again. The gallery can therefore never drift
silently from the product it shows.

## The look

The page follows the verdict-viewer standard David set for
the frontend flex work. A dark header, scene cards on a light
ground, the Okabe-Ito accent the renderer itself uses, system
type, and room around each picture so the diagrams carry the
page. No script, no framework, nothing that moves. The frame
stream at the head is the one moving piece and it moves the
way the product does, whole frames replacing whole frames.

## Fences

The fences from the roadmap stand. The gallery adds no
datastore, no query language, no editor, no server, and no
new product surface. The test that assembles it is a build
tool for one committed page, not a feature. The README gains
one line on its first screen linking the page, and no other
README text changes in this round.
