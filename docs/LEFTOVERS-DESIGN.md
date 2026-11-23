# Round 10 design: the named leftovers

Status: design record for Round 10. Rounds 8
and 9 each closed with named leftovers recorded
in DECISIONS rather than fixed, because each
sat outside its round's fences. This round
takes all three: they are small, they are
already understood, and a record that names a
defect without a round that closes it is only
half a record.

## The three leftovers

The MCP dedup gap. Round 9's combined set made
the CLI count each span once; the MCP tools
still parse their traces argument as it
arrives, duplicates included, so on a
duplicate-carrying export the CLI and the MCP
surfaces report different counts for the same
input. Round 9 named widening the dedup to the
MCP tools as a candidate rather than taking it
inside a CLI round.

The hand-built graph panic. service_layers
indexes its component map by the names in the
graph's edges. A ServiceGraph built by hand,
with an edge naming a service absent from its
nodes, can panic or silently absorb the unknown
name. service_graph cannot produce such a
graph, but service_layers is public API and
the derive layer's other public functions
refuse or tolerate malformed input by name.

The flame builder's depth. flame_from_spans
builds its tree by recursion. Round 8 bounded
the recursion by distinct span ids, which
closed the cycle case; a very deep acyclic
trace, a chain of ten thousand spans, can
still overflow the stack. The scale suite
already generates such chains for other
derivations.

## The finish

One dedup rule on every surface. The MCP
input assembly dedupes its parsed spans by
trace id and span id, first occurrence kept,
the combined-set rule from Round 9, so every
MCP tool composes, ranks, and renders over the
same set the CLI builds from the same export.
The rule lives where the inputs are built, so
no tool can opt out.

Edges name services or are ignored.
service_layers builds its adjacency from the
edges whose endpoints both stand in the
graph's nodes; an edge naming an absent
service takes no part in the layering. The
function's output for any graph service_graph
produces is unchanged, and a hand-built graph
gets layers for its named services instead of
a panic.

The flame builder iterates. flame_from_spans
assembles its tree with an explicit stack in
place of recursion, keeping the build-once
rule for repeated ids and the exact tree
shape, values, and self times it produces
today. A chain of ten thousand spans builds
the same tree the recursion built where the
recursion survived, and builds it where it
did not.

## Pins

An MCP test hands the incident tool a traces
export carrying a duplicated error span and
asserts the composed headline counts it once.
A derive test layers a hand-built graph whose
edge names an absent service and asserts the
named services' layers. A derive test builds
the flame tree for a generated chain of fifty
thousand spans and asserts its depth and root
value. The golden corpus and the scale hashes
stand unchanged, which is the proof that
well-formed output did not move.

## Fences

No new panel, dialect, or surface. No output
change on duplicate-free inputs on any surface:
the dedup removes nothing, the edge rule
ignores nothing service_graph can produce, and
the iterative builder returns the same tree.
No change to the CLI, whose combined set
already applies the dedup rule this round
extends.

## Amendments, 2026-11-23

The reviews qualified three sentences above,
and one gap closed inside the round.

The dedup's home is the input assembly the
composing, ranking, and rendering tools share.
The inspect tool parses its own argument on
both surfaces and reports the raw counts on
each, so the parity the finish claims holds
there by symmetry rather than by the shared
rule. The sentence that no tool can opt out
reads too broadly and this amendment scopes
it.

On a duplicate-carrying export the MCP render
tool's panel data now follows its deduped set,
while the CLI render command's panels still
draw the raw traces it was handed, so the two
surfaces part on that malformed class in the
direction of the honest count. The incident
and findings surfaces agree exactly.

The flame scope widened by one function. The
reviews found the renderer's depth verdict was
itself recursive, so a deep flame panel
overflowed inside the verdict before the
48-level refusal could fire. flame_depth is
iterative in this round, with a refusal pin at
twenty thousand spans. Two depth paths remain
outside the round and are named as the next
leftovers: dropping a very deep flame tree
still recurses through the drop glue in the
consumers that own one, and flame_tree, the
folded-stack builder, still inserts and
freezes recursively.
