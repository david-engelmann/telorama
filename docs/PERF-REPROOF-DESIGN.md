# Design record for Round 36, the scale re-proof

## Purpose

Round 14 proved the session side at scale and recorded three figures from the first hosted measurements. Rounds 32 and 34 then changed the path every label takes through the renderer, and no figure has been re-measured since. This round re-runs the same three workloads on current main under a stated method and records what the measurements say. It changes no production code and no test code.

## The Round 14 record

The Round 14 entry in DECISIONS.md carries the figures this round re-proves, and its design record stands at docs/LONG-SESSION-DESIGN.md. One citation precision. DECISIONS records the final frame under a pinned content hash without printing the value. The value c5ed954ad4390b9f is pinned in the frame test source in crates/telorama-svg/tests/golden_tests.rs, and the fences below cite it from there.

- A five thousand turn run through the frame stream built in 13.06 seconds on the first hosted measurement. Its final frame rendered at 4,150,019 bytes under the pinned content hash c5ed954ad4390b9f.
- A ten thousand span GenAI session parsed in 47 milliseconds, asserting turn counts and token totals.
- A ten thousand event Claude Code log parsed in 176 milliseconds, asserting turn counts and token totals.

The record also carries one aggregate caution. The full render at five hundred turns measured 104,428,401 bytes, the total grows with the square of the turn count, and the aggregate at five thousand turns stands on the order of ten gigabytes as a measured estimate rather than a pin. INV-71 keeps that handling and this round does not revisit it.

## Where the workloads live

Three tests carry the whole proof, and all three already run under the normal CI test job, which runs the workspace test suite on the hosted runner on every push. No separate harness exists and the Round 14 record names none. Each test times itself and prints its figure, and each carries a wall clock ceiling at ten times its first hosted measurement under INV-70.

- The frame stream test is frame_stream_at_five_thousand_turns in crates/telorama-svg/tests/golden_tests.rs. It builds the stream, asserts the frame count and the final headline, renders the final frame and a mid-stream frame, and asserts both content hashes. It prints the build time, the final frame hash, and the final frame byte count. Its ceiling stands at 131 seconds.
- The GenAI test is genai_session_parses_ten_thousand_spans in crates/telorama-ingest/tests/ingest_tests.rs. It generates one thousand turns of ten spans each and asserts the turn count, the token totals, and the per-turn emission count. Its ceiling stands at 470 milliseconds.
- The Claude Code test is claude_session_parses_ten_thousand_events in the same ingest test file. It generates two thousand prompts of five events each in one OTLP document and asserts the turn count, the event count, and the token totals. Its ceiling stands at 1,760 milliseconds.

Each workload re-runs under its exact test name with output uncaptured, in this form.

```
cargo test -p telorama-svg --test golden_tests frame_stream_at_five_thousand_turns -- --exact --nocapture
cargo test -p telorama-ingest --test ingest_tests genai_session_parses_ten_thousand_spans -- --exact --nocapture
cargo test -p telorama-ingest --test ingest_tests claude_session_parses_ten_thousand_events -- --exact --nocapture
```

One honest note belongs here. The frame test and its hash assertions have run green in the normal CI job on every push since Round 32 landed, so the byte identity of the final frame at scale is already held by the suite. What no run keeps is the printed figures. CI asserts the ceilings and discards the timings, and a single run is one noisy sample in any case. The re-proof exists to harvest repeated timings under a stated method and to take the byte count from the measurement prints rather than from the hash alone.

## The measurement route

A temporary workflow on a measurement branch follows the pattern of the generation workflows this repository already uses. It is dispatched by hand, runs on ubuntu-latest, the same hosted runner class as the CI job, and lands nothing.

- The job builds the workspace tests once.
- Each sample is one invocation of one test under its exact name with output uncaptured, so no two measured tests share a process and no test threads compete inside a sample.
- The frame test runs five times in a job and each parse test runs seven times.
- The workflow is dispatched three times, so the pools rest on more than one runner instance. The pooled counts stand at fifteen samples for the frame build and twenty-one samples for each parse.
- The harvest takes the printed lines from the job logs. The five printed lines are these.

```
long session stream build
long session final frame hash
long session final frame bytes
long genai session parse
long claude session parse
```

- The reported statistics are the median and the spread of each pool, the spread stated as its smallest and largest samples.
- The measurement branch is deleted when the round closes. Its workflow never reaches main.

## The fences

Fence one, byte identity. Every frame sample must print the final frame at 4,150,019 bytes under the hash c5ed954ad4390b9f, and the test and its own assertions cover the mid-stream hash de36a42155768390 on every run. The ground for expecting identity is narrow and stated. Every name in the stream is plain ASCII, the model gpt-5.1, the session sess-long, and prompt ids built from the letter t and digits, so under the Round 31 width rule every character counts one and the render should not move. Any miss stops the round. A moved frame means the width work leaked into Latin rendering and contradicts the Round 32 byte fence, and that finding is reported rather than recorded past.

Fence two, the parse medians. The median of each parse pool is compared against its Round 14 figure. A GenAI median at or under 94 milliseconds and a Claude Code median at or under 352 milliseconds record as unchanged within runner noise. A median beyond twice its Round 14 figure counts as a regression finding only when the pool is tight, tight meaning a largest sample no greater than one and a half times the median. A wide pool means the runner cannot answer the question, and the dispatch count rises toward five before the round concludes anything. If a parse pool is still wide at five dispatches, the landing entry records the full pool and states the parse timing question unanswered at this precision, with no median recorded as a figure of record, and the round closes on that statement rather than on a number. One boundary is stated in advance. The ingest crate depends on the scene crate and never on the renderer crate where the width code lives, so a real parse regression could not be a width cost, and the report must say so rather than let the timing sit beside the width rounds unexplained.

Fence three, the frame build median. The frame build is the one figure with a real reason to move, because every label in every frame now passes through the width function. A median at or under 19.6 seconds records as the new figure beside the old. The line is one and a half times the Round 14 figure, whose exact product is 19.59 seconds, stated here so the rounding is on the record. A median beyond that line counts as a defect only when the frame pool is tight under the same test as fence two, a largest sample no greater than one and a half times the median. A wide frame pool raises the dispatch count toward five under the same rule as fence two, and a frame pool still wide at five dispatches lands the same unanswered statement fence two names, with the full pool recorded and no median as a figure of record. A defect under a tight pool is a defect report and not a recorded number. A faster median records without ceremony.

Fence four, the ceilings stand. The INV-70 ceilings inside the three tests, 131 seconds and 470 milliseconds and 1,760 milliseconds, are not edited in this round under any outcome. They remain the gross regression gate INV-70 describes, and the finding lines in fences two and three sit far inside them on purpose, because a gate set at ten times a figure cannot speak about a doubling.

## What lands

When every fence holds, one entry lands in DECISIONS.md for Round 36 through the normal branch and pull request route. The entry records each new median and spread beside its Round 14 figure, the sample counts behind them, and the runner class they ran on. Nothing else lands. No new invariant is proposed, because INV-69 and INV-70 and INV-71 already govern the scale proof and this round only refreshes the figures those invariants rest on.

## What this design does not do

- It changes no production code and no test code.
- It does no optimisation work. A slow figure is reported with its evidence, and any fix belongs to a later round.
- It edits no ceiling and reopens no settled question from Rounds 31 to 34.
- It does not materialize the five thousand turn aggregate byte total, which stays a recorded estimate under INV-71.
- It proposes no re-architecture of the frame stream. INV-71 already records its quadratic shape as by construction.
