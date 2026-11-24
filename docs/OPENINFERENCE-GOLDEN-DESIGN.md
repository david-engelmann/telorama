# Design record: the OpenInference golden scene

This record designs a small build round. It exists because the
Round 28 accuracy sweep found a real asymmetry, not because a
larger feature is planned. Nothing in it authorizes the build
on its own.

## The gap

The golden corpus carries an agent run scene for three of the
four session dialects, Claude Code, Codex in two variants, and
GenAI. OpenInference, the fourth dialect, has a recorded
session fixture and adapter coverage, and it renders in the
verdict viewer's recorded run, but it has no scene fixture and
no golden of its own. The README's corpus sentence is honest
about this only by a nuance, recorded in the Round 27 audit,
that the OpenInference fixtures back the adapter and derivation
tests rather than a golden scene. A reader who goes looking
for the fourth dialect's picture does not find one. This round
closes that gap with the smallest artifact that does it, a
scene, a test, and a golden in the exact shape of the GenAI
one.

## The three artifacts

First, a scene fixture at tests/fixtures/scenes/
agent_run_openinference_scene.json, mirroring the GenAI scene
fixture field for field with the OpenInference facts in place
of the GenAI ones. One agent_run panel over one data entry
whose source kind is openinference_session. The context range
covers the session fixture's own window, which runs from
1794855600000 to 1794855609000 in unix milliseconds, padded the
same way the GenAI scene pads its fixture window. The headline
and narrative are written for what the render will actually
show, one turn priced as telorama's estimate, and are checked
against the generated golden before the build is called done,
the same discipline the GenAI scene's narrative follows.

Second, a golden test in crates/telorama-svg/tests/
golden_tests.rs named golden_agent_run_openinference_scene,
cloned from the GenAI test beside it. It parses the new scene,
parses tests/fixtures/agents/openinference-session-spans.json,
adapts it with parse_openinference_session, renders through
render_scene_with_data with the run in place, and pins the
result with assert_golden against tests/golden/
agent_run_openinference_scene.svg.

Third, the golden itself, generated on the hosted route and
never by hand. The generation runs the new test alone with
UPDATE_GOLDENS=1 in a temporary workflow on the build branch,
the same mechanism the viewer record uses, and the workflow
commits the golden back to the branch. The build then re-runs
the full hosted gate with the golden committed, so the pin is
proven against a clean run rather than against the generation
run that produced it.

## What the golden will show

The facts are already measured, because the viewer record
renders this same session. One turn of 2900 input tokens, 570
output tokens, and 600 cache read tokens, priced at 0.0094
dollars on gpt-5.1 as telorama's estimate from its price
table, with the calculator tool call in the run, and a coverage
line counting no log events, five spans read, and five spans
merged. The build verifies the golden carries those figures.
If the generated golden disagrees with any of them, the build
stops and reports rather than pinning a surprise.

## Fences

No production code changes. The adapter, the renderer, and the
scene spec already handle this dialect, and the round proves
it by rendering, not by modifying. The new golden does not
join the gallery page, which shows one render per scene
family, and the agent run family is already represented. Two
counts elsewhere move and the build moves them in the same
change. The README's golden corpus count goes from thirteen
scenes to fourteen, and the DECISIONS entry for the build
round records the corpus at fourteen light scenes with the
five dark pins unchanged. No invariant is added, because the
round adds a pin inside INV-62's existing golden discipline
rather than a new rule.
