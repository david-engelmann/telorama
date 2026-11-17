# Composing an incident scene

When something is failing and the window is known, the first
scene of the incident should not be hand authored under time
pressure. The incident template composes it from the window's
spans under fixed, stated rules.

```sh
telorama incident --traces tests/fixtures/otel-demo-checkout.json
```

The command prints the composed scene spec as JSON. Add `--out`
to write the spec to a file and `--render` to write the rendered
SVG alongside it. The same composition is available over MCP as
the `incident_scene` tool.

## What the template composes

Four panels, in the order an investigator reads them. A service
map of the window, the RED row across every service, a waterfall
of one exemplar trace with the critical path and errors
highlighted, and a latency heatmap whose bucket edges the
template declares on the data entry. When an agent session
overlaps the window, its run attaches as a fifth panel.

## How it chooses

The failing service is the one with the highest error ratio among
services that recorded an error, with ties broken on more errors
and then on the service name. In the checkout window that is the
email service, one error in one call, a ratio of 100 percent, and
the headline says exactly that.

The exemplar trace is the longest trace, by root span duration,
among traces that contain an error, and the longest trace in the
window when nothing errors. The checkout export holds a single
trace, so the exemplar is the checkout trace itself, and the
waterfall shows its critical path running to the failed email
span.

The headline and the narrative are derived from the same numbers
the panels print, so the words and the picture cannot drift
apart. The composed scene passes the same validator as a hand
authored scene, and rendering it produces
`tests/golden/incident_checkout_scene.svg` byte for byte.
