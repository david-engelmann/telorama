# Round 12 design: the agent tree

Status: design record for Round 12. Round 11
closed the flame tree's depth paths and its
red team sweep found the agent tree still
recursing over input depth in two places.
This round takes both, and gives the workspace
one nesting budget for agent runs.

## The two paths

The dialect session builder's emit recurses.
It walks a turn's span tree and emits agent
steps, recursing once per level. Its seen set
bounds cycles through repeated ids, but an
acyclic chain of nested agent spans recurses
as deep as the export says, and a deep enough
session overflows inside parse.

The agent-run layout walk recurses. The
renderer draws a run's steps by recursing
into each subagent's steps with no verdict
capping the depth. The Claude Code adapter
builds its nesting iteratively, so a session
with very deep subagent nesting constructs
safely and then overflows at render. The run
trees themselves drop and serialize
recursively on the same paths, so an
iterative walk alone does not finish the
property.

## The finish

Emit iterates. The builder walks the span
tree on an explicit stack, each frame holding
its own step sink. A container frame splices
its steps into its parent, an agent frame
wraps its sink in the subagent step, and the
seen rule fires at frame creation exactly
where it fired at entry. The emitted steps
are the same steps in the same order.

The layout walk iterates. The renderer's
agent-step layout becomes a pre-order walk on
an explicit stack, drawing each step's row at
its depth in visit order, the same rows in
the same order the recursion drew.

One nesting budget bounds the trees. No
agent run nests subagents deeper than 512
levels. The budget is enforced where nesting
is built: the dialect builder refuses with a
named error when an agent span would nest
beyond it, and the Claude Code adapter
refuses the same way when its wrapping would.
Five hundred twelve is two orders of
magnitude past any real session's nesting,
and it bounds every remaining recursion over
a run tree, the drop, the serialization, and
any walk a future surface adds, to a depth
the stack provably holds. A session past the
budget is refused whole, never truncated,
because a partial run would misstate the
session it came from.

With both walks iterative and the trees
bounded, INV-87's exception retires for
real this time, and its text states the
budget as the mechanism.

## Pins

An ingest test builds a generated GenAI
session whose agent spans nest past the
budget and asserts the named refusal, and a
companion at the budget's edge builds. The
golden corpus stands unchanged, the agent
run goldens pinning the layout's bytes
through the walk's conversion.

## Fences

No new panel, dialect, or surface. No output
change on any session within the budget: the
iterative emit produces the same steps, the
iterative layout draws the same rows, and
every golden stands. The budget refuses only
sessions no current surface could render
safely anyway.
