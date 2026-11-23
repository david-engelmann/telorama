# Round 13 design, the registry refreshed

Status. Design draft, prepared 2026-11-23 while
the GitHub pool was drained. Lands as
docs/REGISTRY-REFRESH-DESIGN.md in the design PR.

## The gap

The crates on crates.io stand at 0.2.0, published
2026-11-23 in Round 6. Six rounds have landed
since. Round 7 documented the whole public
surface under a deny gate. Round 8 made every
derivation terminate on cyclic input, taught
the renderers refusals, and normalized dated
model ids. Round 9 completed CLI and MCP
parity. Rounds 10 to 12 finished the depth law,
so no derivation, verdict, walk, or drop in the
workspace recurses over the depth of its input,
and agent runs carry a stated nesting budget.
None of that is on the registry. A reader who
installs telorama today gets the pre-hardening
build, and the registry readmes still describe
the 0.2.0 state.

## The finish

All five crates version together at 0.3.0, a
minor bump for a hardened and documented
surface with no breaking change, and publish in
the settled order, scene, ingest, svg,
telorama, telorama-mcp, each version confirmed
live before the next is sent (INV-79). The
workspace dependency pins move with the
versions and Cargo.lock is regenerated in the
packaging pass and committed, the Round 6
mechanics unchanged.

The root readme's Current state paragraph is
rewritten for the finished state. It names the
hardening plainly, the documented surface, the
cycle and depth law, the nesting budget, and
the version. The five crate readmes gain the
same version sentence where each states the
project's state. No other copy changes, and the
root readme keeps its relative media (INV-78).

The packaging tests already pin version
agreement, workspace pins, and readme shape, so
they carry the round without new tests. The
publish runs through the settled route, package
on a CI runner, publish in order with the
stored credential, verify each version live,
then read the rendered 0.3.0 crate pages and
the docs.rs pages firsthand and verify the
media and the worked examples.

## Fences

No behavior changes, no new public API, no
golden, fixture, or figure changes. If the
packaging pass convicts a defect, the round
stops and the defect gets its own round.

## Standing lanes

The convention check for this round is the
2026-11-23 charges pass, which re-pinned the
GenAI dialect reference to a named commit and
verified MCP Apps conformance against the
stable spec the same day this design was
drafted. The A2UI watch stands as a watch item,
reviewed and not built on, per the roadmap.
