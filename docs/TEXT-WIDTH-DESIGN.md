# Design record: honest text width for wide and combining scripts

This record designs the mitigation the DOSSIER has carried as
an open spike since the research programme, in its open risks.
Latin-subset fonts fail CJK and Arabic service names. The
spike is now taken up as a design, and this record is the
design. Nothing in it authorizes the build on its own.

## How the failure works today

The renderer's layout law is stated in its own module doc.
Layout uses fixed columns and character counts rather than
measured text, so the output is byte-identical on every
machine without a font dependency. Every label budget in the
renderer is therefore a character count. The shared
truncation helper cuts a label at a character budget, and the
width arithmetic multiplies a character count by a per
character advance tuned for Latin glyphs.

A CJK character occupies roughly two Latin columns in the
fonts these pages are read in, and it counts once. A service
name in Chinese or Japanese is therefore budgeted at half
the width it will actually take, so it overflows the column
the layout reserved for it instead of truncating where its
Latin neighbour would. A combining mark occupies no width
at all and also counts once, so a name carrying accents as
combining sequences is budgeted wider than it renders and
can be cut in the middle of a sequence. The model is honest
for the Latin text it was tuned on and wrong by construction
for the scripts the DOSSIER names.

## The design

The build adds one pure function to telorama-svg that reads
a string's layout width from the classes of its characters.
A combining mark and the zero width characters count zero.
A character in the East Asian wide and fullwidth ranges
counts two. Everything else counts one. The ranges are
hand-derived, written into the code as named ranges with
their sources stated, and pinned by unit tests character by
character at the range edges. No font is consulted, no
dependency is added, and the function is a pure function of
the string, so the determinism law stands untouched.

Truncation and budgeting then work in width instead of
character count. The truncation helper takes a width budget
and cuts at the last character boundary that fits, never
inside a combining sequence, with the ellipsis counted in
the budget as it is today. Each call site keeps the pixel
budget it already computes and converts through the width
function rather than through a raw count. The build's first
task is a census of every site that turns a character count
into a layout decision, and the census is part of the
build's record, because a site the census misses is a site
that keeps the old failure.

## The fence that proves it

For any string whose characters all count one and which
carries no combining marks, the width function returns
exactly the character count the layout uses today. Latin
layout therefore cannot move. Every existing golden, the
gallery page, and the viewer record must pass byte for byte
unchanged in the build, and that is the build's first gate.
If a golden moves, the build has broken the fence and stops.

## What this does not claim

The fix closes the measurable failure, names budgeted and
truncated by a width model that is wrong for wide and
combining scripts. It does not shape Arabic text and it does
not reorder bidirectional runs. Shaping and direction
belong to the text engine that renders the SVG, which joins
Arabic letters and orders mixed runs by its own rules, and
a layout library that pretended otherwise would be claiming
a fix it had not made. Arabic names gain honest budgeting
and truncation that never cuts a combining sequence, and
the record states the rest as the text engine's province.

## The proof the build carries

A new trace fixture whose services carry CJK names, one of
them long enough to force truncation, an Arabic name, and a
name with combining marks, with a golden scene over it that
shows the names inside their columns. Unit pins for the
width function at the edges of every range it names. A new
invariant, INV-92, stating that layout width is a function
of character classes and that truncation never splits a
combining sequence. The golden corpus count moves only if
the new scene joins it, and the build records that count
the same way Round 30 did.

## Why this arc, from the records

The seat audited every item the records still name as open
before choosing. The dated snapshot model id normalization
closed under INV-84. The Claude Code api_refusal convention
closed under INV-83. The CLI session findings divergence
closed under INV-81. The A2UI emitter stays behind its own
fence, because the host that would reopen it has not
appeared. MCP Events stays on the Round 26 watch, because
neither of its triggers has fired. The text width spike is
the one named item that is open, unbuilt, and buildable
inside the fences the programme already keeps, so it is the
arc.
