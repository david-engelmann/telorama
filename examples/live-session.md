# Stepping through a session live

A finished session reads as one run. A live session reads as a
stream of frames, one complete scene per turn boundary, where
frame k shows the session only up to turn k. The frames command
builds that stream from the same exports the session command
reads.

```sh
telorama frames \
  --logs tests/fixtures/agents/claude-code-session-logs.json \
  --spans tests/fixtures/agents/claude-code-session-spans.json \
  --out frames.jsonl \
  --html framebook.html
```

The output file carries one JSON payload per frame. Each payload
names its frame number, how many turns it covers, the frame's
scene spec, the findings the investigation layer derives from
that prefix, the rendered SVG, and the render's content hash.
The demo session has two turns, so the stream has two frames.
Frame 1 closes after the first turn at 3000 input tokens, 750
output tokens, and $0.0270 so far, and its coverage counts only
the events that turn folded.

Each frame is a pure function of its prefix. Cut the export
after turn 1 and rebuild, and the single frame that comes out
equals the first frame of the full stream in its run, its
scene, its findings, and its rendered bytes. Nothing from a
later turn can leak into an earlier frame, because the frame's
coverage is the sum of its own turns' coverage and its pricing
names only the models its own turns used.

The framebook is a static page that stacks the frames under
their captions. It carries no scripts and nothing in it moves.
It is the offline form of the same stream the MCP view steps
through.

Over MCP, the `session_frames` tool builds the same stream from
the same arguments. Its text answer gives one summary line per
frame for the model. Its structured content carries each frame's
scene and findings. The final frame rides the usual view
payload, and every frame rides alongside it, so the view opens
on the completed session and its stepper walks back and forth
through the delivered frames by replacing the picture whole.

Findings ride every frame. A frame's findings derive from that
frame's prefix alone, so the ranked list an investigator reads
at turn 1 can only cite what had happened by turn 1.
