# telorama-scene

The scene spec, its JSON schema, the honesty validator, and the pure derivations of the telorama visualization engine. A scene is a small structured document an agent writes to ask for a diagram. This crate defines what a valid scene is, refuses specs that would mislead, such as a percentile reported as an average, a sankey whose flow does not conserve, or a heatmap without real buckets, and derives every fact the panels print from the trace data itself, including the ranked investigation findings.

Correctness lives in this crate. It performs no I/O and renders nothing, so every public function is a deterministic function of its inputs, and the golden corpus can pin its behavior byte for byte.

The project lives at https://github.com/david-engelmann/telorama, where the gallery shows every scene family rendered from the test fixtures.

telorama-scene is available under the MIT licence or the Apache License, Version 2.0, at your choice.
