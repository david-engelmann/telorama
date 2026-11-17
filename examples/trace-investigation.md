# Investigating one trace

The checkout fixture is a nine-span trace of a single request, a
frontend root, a checkout span, five service calls beneath it, and
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
