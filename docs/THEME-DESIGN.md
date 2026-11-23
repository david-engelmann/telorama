# The dark theme, Round 5 design record

Settled 2026-11-23 before the build, under Round 5 of
docs/ROADMAP.md. The roadmap's charge is a scene-level
theme choice with the same layout and the same figures,
an Okabe-Ito-derived palette with per-pair contrast
checks, byte-determinism per theme, and goldens for both
themes on the flagship scenes. The light theme stays the
default and its bytes do not change.

## Where the choice lives

The scene spec gains an optional theme field at the top
level, beside the scene version. Its values are light
and dark. Absent means light, so every scene written
before this round renders exactly as it does today. The
validator accepts the two values and rejects any other
with a named error. The theme is part of the scene, so
a render stays a pure function of the scene and its
data, and the determinism law needs no new machinery.

Scenes the tools generate take the theme from their
callers. The frame builder and the incident composition
gain a theme argument that defaults to light, so the
existing frame and incident goldens stand untouched.
The command line and the MCP tools pass a caller's scene
through with its own theme intact, and their generated
scenes stay light unless a later round gives their
callers a theme switch. That switch is deliberately not
in this round. The round builds the theme itself, its
palette, and its proofs.

## What the theme governs

The theme governs every color the renderer chooses. The
service palette, the error color, ink and muted text,
grid and border strokes, panel surfaces, the heatmap's
empty cell and its shading base, the flame's border
stroke, and the label color chosen by luminance all
resolve through one theme record the render dispatch
carries. Colors a scene declares for itself are data,
not chrome. Field thresholds carry their own colors in
the spec, and those pass through unchanged in both
themes, because restating an author's stated verdict
color would change what the figure says.

A bare SVG carries no background rectangle in either
theme, which is the current light behaviour and stays.
The theme therefore travels with the drawing's own
colors, and the HTML export paints the page background
to match the scene's theme, white for light and the
dark surface for dark. The gallery page is a light
artifact and is not rebuilt by this round. Its freshness
gate covers light renders, and a dark gallery is a
separate decision if it is ever wanted.

## The dark palette

The dark background is a near-black neutral, and the
panel surface sits one step above it. Ink becomes a
near-white, muted text a mid gray that still clears the
text floor on the background, and grid strokes a quiet
gray visible on the surface without competing with
data. Each Okabe-Ito entry is lightened only as far as
its dark pair requires. The yellow already works on
dark and stays. The black entry cannot appear on a dark
ground and is replaced by a pale gray. The vermillion
error color lightens one step. The heatmap's empty cell
becomes the panel surface, and its cells shade toward
the same sky blue the light theme uses, so a heatmap
reads as the same figure in both themes.

Contrast is a checked property, not an intention. Every
foreground and background pair the renderer can emit is
held to a floor. Text pairs clear four and a half to
one against the surface they sit on. Data marks clear
three to one. A test computes the WCAG ratios from the
palette constants themselves and fails on any pair under
its floor, so a future palette edit cannot silently
drop a pair below the line.

## The sameness proof

Same layout and same figures is enforced mechanically.
A test renders flagship scenes in both themes,
normalizes every color attribute out of both outputs,
and asserts the remaining bytes are identical. Geometry,
text content, and ordering cannot drift between themes
without failing that test. Byte-determinism per theme
follows from the existing rule that a render is a pure
function of its inputs, and the golden corpus gains dark
goldens for the flagship scenes, the agent run, the
incident composition, the checkout waterfall, and the
sankey, generated on a hosted runner by the established
route. Every light golden stands byte-identical, which
is the regression pin for the whole round.

## Fences

No layout changes and no figure changes in either
theme. No new panel, dialect, or surface. No theme
toggle in the MCP view or the gallery in this round.
The spec field and the renderer support are the
deliverable, with the generated-scene plumbing at its
light default.
