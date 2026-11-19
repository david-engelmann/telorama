# Worked examples

Four walkthroughs, each runnable from the repository root against
the fixtures the test suite uses. The fixtures are realistic and
synthetic, a nine-span checkout trace, a small API trace, and
agent session exports in four dialects, and the golden renders in
`tests/golden/` are the expected output of the scene examples, so
anything you produce can be checked against them byte for byte.

- `trace-investigation.md` follows one trace from inspection to a
  rendered scene with a waterfall, a stat, and the RED row.
- `agent-session.md` reads agent sessions in the Claude Code,
  Codex, GenAI, and OpenInference dialects and explains the
  coverage and pricing lines on the run.
- `incident.md` composes the incident template over the checkout
  export and shows how the template picked its service, its
  exemplar trace, and its headline.
- `live-session.md` builds the frame stream for the demo session
  and shows why each frame depends only on its own prefix.

Build the CLI first, or run it through cargo.

```sh
cargo build -p telorama
```
