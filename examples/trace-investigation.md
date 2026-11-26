# Investigating one trace

The checkout fixture is a nine-span trace of a single request, a
frontend root, a checkout span, six service calls beneath it, and
an email span that errors at the end. This walkthrough inspects
the export, validates a scene written against it, and renders the
scene.

## Inspect the export

```sh
telorama inspect tests/fixtures/otel-demo-checkout.json
```

The summary reports nine spans across eight services in one
trace, with one error, the email span. Inspection is the cheap
first move, it tells you whether the export is what you think it
is before any scene is authored against it.

## Validate the scene

```sh
telorama validate tests/fixtures/scenes/checkout_scene.json
```

The scene names its data entry, a window, provenance, and three
panels, a waterfall over the trace, a p95 duration stat, and the
RED row for the window. Validation enforces the honesty rules on
the raw spec. A stat must name a percentile and can never ask for
an average, a heatmap must declare its bucket edges on its data
entry, and provenance is required, so a scene that would print an
unearned number fails here rather than rendering.

The contract validation checks against is printable. The
`schema` command writes the scene spec's JSON schema, so an
agent about to write a spec can read the exact shape first
rather than guess it.

```sh
telorama schema
```

## Render it

```sh
telorama render --scene tests/fixtures/scenes/checkout_scene.json \
  --traces tests/fixtures/otel-demo-checkout.json \
  --out checkout.svg
```

The output is byte-identical to `tests/golden/checkout_scene.svg`.
Reading the render top to bottom, the headline states the finding,
the waterfall shows the critical path through the frontend root,
the checkout span, and the email span, with the email span
carrying its error marker in text as well as color, the stat
reports the p95 span duration of 390 milliseconds, and the RED
row shows the email service at a 100 percent error ratio over its
single call. The caption at the foot states the window, the data
entry, and the provenance, so the picture carries its own
reproducibility.

For a standalone page instead of a bare SVG, swap `render` for
`render-html` and the output is a self-contained HTML document
with the same scene inside it.

## Render a profile

The same window also carries a CPU profile in folded stack
form, one stack per line with its sample count at the end.
The profile fixture holds seven stacks and 970 samples in
all, and the flame scene declares that count, so validation
checks the profile against the declaration before anything
renders.

```sh
telorama render --scene tests/fixtures/scenes/flame_scene.json \
  --profile tests/fixtures/profiles/checkout-cpu.txt \
  --out flame.svg
```

The output is byte-identical to `tests/golden/flame_scene.svg`.
Reading the graph, the payment charge subtree holds 620 of
the 970 samples, and the panel prints it at 63.9 percent of
the whole. The foot of the panel states the count it drew
from, 970 samples of cpu from 7 folded stacks, so the picture
again carries its own evidence.

## Stage the flow

A third reading of the same trace stages its spans by call
depth instead of laying them on a timeline. The sankey scene
names its residual bucket in the spec, the bucket a span
enters when its parent is missing from the data or its
ancestry cycles, so an unbalanced flow shows its gap in the
open rather than staging the span as a root.

```sh
telorama render --scene tests/fixtures/scenes/sankey_scene.json \
  --traces tests/fixtures/otel-demo-checkout.json \
  --out sankey.svg
```

The output is byte-identical to `tests/golden/sankey_scene.svg`.
The nine spans stage one at the root depth, one beneath it,
six at the next depth, and one nested a level deeper. Every
parent in this trace stands in the export, so the residual
bucket the spec names, other, carries nothing here, and no
residual flow is drawn.
