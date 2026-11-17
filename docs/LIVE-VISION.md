# Telorama: the live observability vision (David, 2026-11-16 evening)

David's direction, in his words in substance: think about how a human uses the
results of these tools to find the best places to adjust or investigate, and
how agents can do the same, live-updating the human in a visualization that
updates live with what the agent is doing and why. The ability to create
animated, dynamic, stunning front ends quickly is leverage here: a new type
of monitoring and visual observability, one that forces the agent into using
the tools we know will help it be better.

## The synthesis (Main Muse, for the design record)

**Two readers, one scene.** A human investigator and an agent investigator
consume the same derived truths (critical path, self-time, RED per edge,
thresholds as verdicts). Nothing about the truth layer forks between them.
The human gets the rendered scene; the agent gets the structured payload of
the same scene. Same ranking, same evidence, two renderings.

**Investigation is a first-class layer, not something the reader eyeballs.**
The renderer already derives where time and errors concentrate. Scenes carry
an investigation layer: ranked look-here-first markers, each with its
evidence attached (the exemplar spans, the threshold crossed, the delta from
baseline). A human scanning a waterfall sees where to adjust. An agent
reading the payload receives the same ranked list as data and can act on it
without re-deriving it.

**Live means a stream of honest frames.** Live mode is a sequence of
complete scene frames, each frame deterministic and byte-identical for its
input window. Frames replace; values never interpolate or tween. This keeps
the determinism the whole design stands on and keeps house law on live
surfaces (payload data never animates, numbers never transition) while the
front end around the frames can be as dynamic and polished as Stageshow
craft allows. The stunning part is the shell, the framing, and the demo
material; the data stays exact.

**The agent narrates into the scene.** The working agent reports its own
state as structured input: what it is doing, why, what it is waiting on,
what it will do next. The scene spec gains an agent-state panel family for
this, rendered like any other panel. The blocked-on-user segment in Claude
Code's span tree is the precedent: time spent waiting on the human is a
first-class part of the picture, not a gap in it. A person watching the live
view sees the run and its reasons, not just its timings.

**Observability as a forcing function.** The live view is only producible by
calling telorama's tools, and scene validation requires the parts that make
an agent better: a rationale on the agent-state panel, exemplar links on
investigation markers, derived truths instead of raw dumps. An agent that
uses telorama to report itself is automatically pushed into the
investigations and explanations that help it, because the tool will not
render the lazy version. The visualization steers the behavior, not just
displays it.

## Scope placement

This extends the dossier; it does not replace it. The deterministic frame
renderer (M1) is the foundation live mode stands on, so live frames,
agent-state panels, and the investigation layer scope as their own milestone
after the static core, recorded by the seat as a design addendum. The fences
hold: no datastore (frames render from supplied or streamed telemetry,
nothing is stored), no query language, no dashboard editor, no alerting, no
hosted renderer.
