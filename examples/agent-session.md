# Reading an agent session

Agent sessions arrive in four dialects. Claude Code and Codex
export OTLP logs, and GenAI and OpenInference describe the run as
spans. The `session` command adapts any of them into one canonical
run model and prints it as JSON.

## Claude Code

```sh
telorama session \
  --logs tests/fixtures/agents/claude-code-session-logs.json \
  --spans tests/fixtures/agents/claude-code-session-spans.json
```

Logs carry the turns, the model calls, the tokens, and the cost,
because Claude Code exports cost itself. The spans fuse in what
logs cannot say, how long each tool call waited on a permission
decision and which calls were subagents. The run's coverage block
reports the fusion honestly, eleven log events read, six spans
read, two spans merged into steps.

![The demo session rendered as a run diagram](<../docs/media/agent-run.png>)

## Codex

```sh
telorama session \
  --logs tests/fixtures/agents/codex-session-logs.json \
  --dialect codex
```

Codex exports tokens and no cost, so the run is priced by
telorama from its own table and the run carries a pricing block
naming the table date and the models it priced. The adapter also
normalizes Codex's gross input figure, which counts cached tokens
inside the input total, into the canonical non-cache input, so a
Codex run and a Claude Code run can be compared line by line.

## GenAI and OpenInference spans

```sh
telorama session \
  --spans tests/fixtures/agents/genai-session-spans.json \
  --dialect genai
```

```sh
telorama session \
  --spans tests/fixtures/agents/openinference-session-spans.json \
  --dialect openinference
```

Span dialects build the run from the span tree. Agent and
workflow spans are structure, model and tool spans are steps, and
everything else is a transparent container whose children still
walk. The GenAI adapter reads the union of the attribute shapes
across the convention's generations, so a span exported before
the provider attribute was renamed maps the same way as one
exported after. Cost is priced from the shared table, as with
Codex, and a model the table does not know is named in the
pricing block as unpriced, with its tokens counted and its cost
excluded, never folded in as zero.

## Putting a run in a scene

A session becomes a picture through an agent_run panel. The
scenes in `tests/fixtures/scenes/` named for each dialect render
through the same `render` command as any other scene, with the
session export passed alongside the scene.

```sh
telorama render \
  --scene tests/fixtures/scenes/agent_run_codex_scene.json \
  --session-logs tests/fixtures/agents/codex-session-logs.json \
  --dialect codex --out codex-run.svg
```

The rendered run prints its coverage and pricing lines on its
face, so a reader can see where every figure came from without
leaving the picture.
