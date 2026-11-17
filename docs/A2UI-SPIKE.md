# A2UI emitter spike

Status, watch item only. This note scopes a possible future
emitter. It is not a commitment, not a design, and not part of
any milestone's landed scope. It exists so the question has a
written home and the fence around it is explicit.

## The question

Telorama renders a validated scene spec to SVG, to a standalone
HTML page, and to the MCP Apps view. A2UI is an agent-to-user
interface protocol in which an agent describes interface elements
declaratively and a host renders them natively. The spike
question is whether a scene should also emit an A2UI form, so a
host that speaks A2UI could render a scene's panels as native
elements instead of receiving a picture of them.

## What an emitter would have to map

Every panel type would need a counterpart element or an honest
degradation. Tables and stat values map naturally. The waterfall,
the service map, the flame, and the heatmap are drawn geometry,
and their A2UI form would either be a canvas-like element, which
not every host offers, or a structured restatement, which loses
the picture that is the product. The triple contract would also
need a fourth leg, and the determinism law would need a ruling
on whether byte identity can even apply to a form a host renders
with its own components.

## Why it waits

The scene model already carries the full contract, data,
derivations, provenance, and the honesty rules, in a form any
future emitter can target without changing a scene. The MCP Apps
view covers the hosts telorama targets today. An A2UI emitter
earns its build when a host telorama targets renders A2UI and
asks for scenes in that form, or when the protocol's treatment of
drawn geometry settles. Until one of those happens, the spike
stays behind this fence and the static core does not plan around
it.

## What would reopen it

A named host with A2UI rendering and a reason to prefer native
elements over the MCP Apps view for telorama's panels. At that
point the spike becomes a design exercise against that host's
element catalog, costed like any other emitter.
