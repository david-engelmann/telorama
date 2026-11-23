# Round 11 design: depth, finished

Status: design record for Round 11. Round 10
closed the flame builder, the depth verdict,
and the renderer's ownership of its tree, and
named the two depth paths that remain. This
round takes both, and retires the exception
clause Round 10 had to write into INV-86.

## The two paths

The folded-stack builder still recurses.
flame_tree folds folded stacks through a
builder whose insert recurses once per frame
in a stack and whose freeze recurses once per
level of the tree it built. A single stack
carrying tens of thousands of frames, accepted
as it arrives by the profile parsers, overflows
on either recursion.

The MCP panel paths still drop deep trees.
panel_summary and panel_detail build the flame
tree for their panel, read its top level, and
drop it. The drop recurses through the glue, so
a deep trace behind a flame panel aborts the
tool on the way out, the failure Round 10's
renderer learned to dismantle against.

## The finish

One dismantle, in the type's own crate.
telorama-scene gains a public
dismantle_flame_tree that takes a flame tree
apart level by level. The renderer's private
helper is replaced by it, and the MCP panel
summary and detail paths dismantle the tree
they built once they have read it. No consumer
that owns a flame tree drops it whole.

The folded-stack builder iterates. Insert
walks the frame path in a loop instead of
recursing per frame. Freeze assembles the
frozen tree on an explicit stack, producing
the same tree the recursion produced: the same
values, which insert accumulates, and the same
child order after the same sort.

The exception retires. With both paths closed,
INV-86's headline no longer needs its
exception clause, and the build record removes
it. INV-87 states the finished property: no
derivation, verdict, walk, or drop in the
workspace recurses over the depth of its
input.

## Pins

A derive test folds one stack of fifty
thousand frames through flame_tree and asserts
the root value and the frozen tree's depth. An
MCP test asks the panel detail tool for a
flame panel over a folded profile whose stack
runs twenty thousand frames deep and asserts
the frames it returns. The golden corpus and
the scale hashes stand unchanged, the proof
that ordinary output did not move.

## Fences

No new panel, dialect, or surface. No output
change on any input the current code handles:
the iterative insert and freeze return the
same tree, and dismantling happens after the
values are read. No behavior change in the
renderer beyond calling the shared dismantle
where its private one stood.

## Amendments, 2026-11-23

The build's red team swept the workspace
against INV-87's letter and found two paths
outside this round's scope that still recurse
over input depth. The dialect session
builder's emit recurses over an acyclic span
tree's depth (its Round 8 guard bounds cycles
only), so a session export carrying a chain of
tens of thousands of nested agent spans
overflows inside parse. The agent-run layout
walk recurses over a run's nesting depth with
no verdict capping it, and the run trees
themselves drop recursively on the same
paths. Both predate this round and neither is
a regression from it. INV-87 as landed names
both as the Round 12 leftovers instead of
claiming the finished property outright, and
the exception retirement above applies to
INV-86 only. The MCP pin was also raised from
twenty thousand frames to fifty thousand, the
depth at which a deep tree's drop is proven to
overflow, so the panel paths' dismantles are
pinned by a test and not by reading alone.
