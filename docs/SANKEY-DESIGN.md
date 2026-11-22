# Round 1 design: the sankey renderer

Status: designed 2026-11-22, before build, under the roadmap in
docs/ROADMAP.md. This record settles what a sankey panel means in
telorama, how its flow is derived, and how it is drawn. The
invariants it adds stand in docs/INVARIANTS.md as INV-58 to
INV-61.

## What the panel shows

A sankey panel shows how work flows through the stages of the
traces it reads, counted in spans. The columns are call depth:
stage zero holds the spans that begin a trace, stage one the
spans they directly caused, and so on. A node is one service at
one stage. A link is the parent-to-child step between two spans,
aggregated between the nodes those spans sit in, and its weight
is the number of spans that made that step. The reader sees
where volume enters, how it fans out or narrows through the
stages, and where it ends.

## The derivation

The unit of flow is one span, and every span in the panel's data
set is counted exactly once, in exactly one node. Depth is
computed within each trace by walking parent links. Three cases
decide where a span's unit enters:

- A span with no parent in the data is a root. Its unit enters
  at stage zero in its own service's node.
- A span whose parent is in the data links from its parent's
  node, one stage deeper, whatever service either side belongs
  to. Same-service steps are real flow and are drawn. Dropping
  them would break conservation and hide depth.
- A span whose parent is named but absent from the data, or
  whose ancestry cycles back on itself, cannot be staged on
  evidence. Its unit enters through the residual node, a single
  stage-zero source named by the panel's residual field, and
  links from there into the node's own service at stage one.
  Its descendants stage normally beneath it.

Conservation then holds by construction. For every node, the
inflow plus the root spans staged there equals the node's
value, and the outflow is the value minus the spans that end
at that node. The residual node's outflow is exactly the count
of spans that could not be staged on evidence, and the panel
prints that count under the diagram, naming the bucket,
whenever it is nonzero. When it is zero, no residual node is
drawn. The derivation lives in telorama-scene beside the
service map derivation and follows its evidence discipline.

## The drawing

Nodes in a stage stack in a fixed order, value descending and
then service name ascending, so the same data draws the same
picture regardless of the order spans arrived in. A node is a
block whose height is its value on a scale set by the tallest
stage, labeled with its service name and its count. Links are
filled trapezoids from the right face of the source block to
the left face of the target block, stacked in node order at
both ends, filled with the source service's color at reduced
opacity. The residual block and its links are gray. The
DisplayList gains a polygon primitive to carry the trapezoids.
every existing primitive keeps its current bytes.

Sub-pixel slivers are a known limit of exact geometry at large
volumes: a one-span node among ten thousand is thinner than a
pixel, and no minimum height is faked, because a faked height
would misstate the flow. Round 4 of the roadmap owns behavior
at that scale.

## The surfaces

The renderer lands in telorama-svg, so the CLI inherits it.
The MCP structured figures for a sankey panel report its nodes,
its links, and its residual count in place of the standing
note that no renderer exists, and the panel detail tool does
the same. The golden corpus gains a sankey scene over the
checkout fixture, and the derivation gains tests over
hand-built span sets for the root, same-service, orphan, and
cycle cases. The golden test that used the sankey panel as its
example of an unsupported type retires with the gap it was
guarding. The guard arm in the renderer stays for panel types
the spec may gain later. The README's sentence about the
missing renderer comes out in the landing.
