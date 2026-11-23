# Round 7 design: the documented surface

Status: design record for Round 7, the first round
after the six-round roadmap. It takes up the two
items the registry round's reviews left named in
the open: the undocumented public surface, and the
CLI session-findings divergence that has stood
since Round 2.

## The recon

A missing-docs deny was tried on the four library
crates during Round 6's repair pass and convicted
176 undocumented public items: 138 struct fields,
34 enum variants, 3 methods, and 1 associated
function, nearly all of them in telorama-scene.
The module and function documentation is in good
shape. What is missing is the field-by-field and
variant-by-variant layer, which is exactly the
layer a reader of the docs.rs pages meets when
they open a struct such as AgentRun or Scene and
find the fields bare.

Separately, the findings wiring differs by
surface. The MCP render path and the gallery
compute findings over a session's spans plus its
agent run. The CLI render command computes them
over its trace spans only, and the session it
parses from logs never contributes spans, so a
CLI render of the flagship Claude Code session
draws no span-based findings where the same
session through MCP carries them. The divergence
was found in Round 2 and recorded as open in
every round since.

## The finish

Documentation completeness. Every public item in
the four library crates carries a doc comment,
and each crate root denies missing docs, so the
gate is permanent rather than a one-time sweep.
A field doc states what the field holds, with its
unit or sentinel meaning where one exists, such
as unix milliseconds, a count of spans, or an
empty string standing for none. A variant doc
states when the variant occurs. A doc that merely
restates the item's name in other words does not
count, and the reviews check for it the way they
check tests for tautology. The CLI crate is a
binary with no public surface and stays out.

Findings parity. The CLI session path computes
its findings over the session's spans plus the
agent run, the same inputs the MCP render path
uses, so every surface that renders a session
ranks the same findings. A test pins the parity
on the demo session: the findings the CLI reports
for the Claude session are the findings the MCP
session path reports for it. The CLI's session
renders gain the findings strip where the inputs
convict something. That is a deliberate contract
repair, recorded here rather than slipped in, and
it closes the Round 2 divergence.

## Fences

No renderer, layout, or figure changes beyond the
findings strip appearing on CLI session renders
whose inputs convict. No new public API; the
round adds documentation and one wiring change.
No golden changes; the library corpus carries no
tool strip by the standing contract, and the
parity test compares findings, not bytes.
