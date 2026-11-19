# Live observability design (M5)

This record scopes the live observability milestone from the terms of
docs/LIVE-VISION.md. The vision holds five commitments. Two readers
consume one scene. Investigation is a first-class layer of ranked
markers with evidence attached. Live means a stream of complete
deterministic frames that replace each other. The working agent
narrates its own state into the scene, with a rationale the validator
requires. The live view is only producible through telorama tools, and
that constraint is the forcing function that makes agents using it
investigate and explain. The fences from the vision hold throughout.
No datastore. No query language. No dashboard editor. No alerting. No
hosted renderer. No streaming server. Frames render from supplied
exports, and nothing is stored.

## The investigation layer

A finding is one ranked look-here-first marker. It carries a kind, a
subject, a statement generated from measured values, the evidence
numbers behind the statement, and at least one exemplar that exists in
the input data. Four kinds exist in this milestone.

- errors names the service carrying the largest share of the errors in
  the window, with its error count, its error ratio, and the window
  error total as evidence.
- latency names the span with the largest self time, with its self
  time, its share of the window self time, and its operation and
  service as evidence. When the service behind it crosses the field
  system's standing verdict threshold for error ratio or duration,
  the crossing is stated as part of the evidence.
- wait names the waiting time inside an attached agent run, with the
  total wait, the largest single wait, and the share of session wall
  time as evidence. It appears only when a run is attached and waiting
  time exists.
- cost names the most expensive turn of an attached run, with its
  cost, its share of the session cost, and its model as evidence. A
  run with unpriced models names them here even when priced cost is
  zero, so the disclosure travels with the ranking.

At most one finding per kind is emitted, so the list never exceeds
four. The order is a total order. Kind precedence runs errors, then
latency, then wait, then cost. The derivation is a pure function in
telorama-scene over the same derived truths the panels print, the RED
rows, self time, the critical path, and run totals. A finding states
evidence and never prescribes a remedy.

Surfaces. The renderer draws the findings as a ranked strip directly
under the scene headline when the caller supplies them. The structured
payload carries the same list under the investigation key, so the
agent reader and the human reader see the same ranking. The CLI and
MCP render paths compute findings automatically on every render that
has span or run data. There is no flag and no argument that suppresses
them, because the forcing function is the point. The library render
stays a pure function of what the caller supplies.

## The agent-state panel

The scene spec gains one panel type, agent_state, a spec change in the
closed panel set. Its data entry carries the source kind agent_state.
The state itself is runtime data, supplied the way spans and runs are
supplied, as a list of entries with an agent label, a current action,
a rationale, an optional waiting-on note, an optional next step, and
an optional start time. Parsing refuses an entry whose action or
rationale is empty. Rendering draws one block per entry, with the
waiting line in the error colour when the agent is waiting, following
the blocked-on-user precedent in the Claude Code span tree. A scene
whose agent_state panel has no state data fails the render the way
any missing data fails it.

## Frames

A frame is one complete scene rendered from a prefix of a session
export. Frame boundaries are turn boundaries. Frame k renders the run
scene for the first k turns, with the run totals and the investigation
findings of that prefix. Every frame validates and renders on its own.
No frame reads data past its own prefix, and no value in a frame is
interpolated from a neighbouring frame.

Surfaces. The CLI gains a frames command that reads a session export
in any supported dialect and writes one JSON payload per frame, one
per line, and can also write a framebook HTML file holding every
frame in order with its caption. The MCP server gains a session_frames
tool under the triple output contract. Its text is one summary line
per frame. Its structured content carries the frame payloads. Its meta
carries the final frame SVG for the view under the standing key and
every frame SVG under a frames key.

## The view

The MCP Apps view learns to step through delivered frames. When a tool
result carries the frames key, the view shows shell chrome with a
previous button, a next button, and a frame position label.
Switching frames replaces the whole SVG in the data container. The
data container carries no transitions. Polish lives in the shell
chrome only, under the standing house law that payload data never
animates and numbers never transition.

## Invariants drafted before build

- INV-49. Findings derive only from the derived truths the panels
  print. A finding's evidence numbers equal the numbers the matching
  derivation produces for the same input.
- INV-50. Every finding carries at least one exemplar that exists in
  the input data. A candidate with no exemplar is not emitted.
- INV-51. The finding order is the total order stated above, and the
  statements are byte-identical for identical input.
- INV-52. A finding states evidence and never prescribes a remedy.
  Statements are generated from measured values only.
- INV-53. An agent-state entry with an empty action or an empty
  rationale is refused at parse. An agent_state panel without state
  data fails the render.
- INV-54. Frame prefix purity. Frame k is a pure function of the
  session prefix through turn k, proven by rendering a truncated
  export and comparing the leading frames.
- INV-55. Every frame is a complete scene that validates and renders
  standalone, and every figure in a frame derives from its own
  prefix only.
- INV-56. The view displays only frames delivered in the tool result,
  and stepping replaces the whole frame in a container that carries
  no transitions.
- INV-57. The CLI and MCP render paths attach findings automatically
  whenever span or run data is present. No flag and no argument
  suppresses them.

## Slices

1. The investigation layer, derivation, render strip, structured
   payload, CLI and MCP wiring.
2. The agent-state panel, spec, parse rule, renderer, golden.
3. Frames, the CLI command and the session_frames tool.
4. The view stepper.
5. Close-out, invariants in formal form, README sync, DECISIONS
   landing record, mirror, gate report.

Each slice lands through a pull request with the hosted gate green on
its exact head, the landing form the repository ruleset now requires.
The three reviews run at the milestone checkpoint before the landing
record is written.
