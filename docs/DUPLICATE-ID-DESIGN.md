# Round 16 design: one id, two spans

## The leftover this round closes

Round 15 recorded one observation without a fix, under the name F7. The investigation's latency finding can misattribute its subject under duplicate span ids, naming one copy's service against another copy's self time, on malformed input only and with correct totals. This round's recon shows the mechanism is wider than the finding. Every surface that joins derived rows back to spans does it by span id, and the row builder itself deduplicates by id, so a duplicated id loses data in three places at once.

Duplicate span ids in one trace are not exotic. The Zipkin data model gives a client span and its server span the same id on purpose, and bridges from that model into OTLP emit both spans under one trace id and one span id, distinguished by kind and service. A renderer that silently drops one copy, or names the other copy against its figures, misreads a shape that real pipelines produce.

## The mechanism, with the code as found

The row builder is `waterfall_rows` in crates/telorama-scene/src/derive.rs. Its walk keeps a seen set keyed by span id, so the second occurrence of a duplicated id never gets a row at all. Its children map is keyed by parent span id, so the children of both copies merge under the shared id and are subtracted from the first copy's self time only. The dropped copy contributes nothing, not even to the totals.

Three consumers join rows back to spans by id, and they do not even agree with each other. The SVG waterfall in crates/telorama-svg/src/lib.rs builds a by-id map with collect, which keeps the last copy, and takes its drawing window from that map's values, so a dropped copy's times can shrink the window as well as misname the bar. The MCP panel detail in crates/telorama-mcp/src/lib.rs finds the first copy with a linear find. The latency finding in crates/telorama-scene/src/investigate.rs collects a by-id map, which keeps the last copy, and names that copy's name and service against a row whose self time came from the first copy's walk. One trace can therefore carry three different answers to the question of which span a row describes.

## The defined rule

Attribution is by occurrence, not by id. Four clauses make that concrete.

First, every span occurrence in a trace gets exactly one waterfall row. The walk tracks occurrences and never suppresses a copy.

Second, a child attaches to the first occurrence of its parent id in canonical order. Canonical order is the trace's existing sort, by start and then span id, with input order as the final tiebreak the stable sort already provides. The children of a duplicated id cannot be told apart, so they hang from the first copy and the later copies carry the self time of their own intervals alone. The rule is arbitrary in the way a tiebreak is arbitrary, and it is stated here so the behavior is defined rather than emergent.

Third, WaterfallRow gains the occurrence's index into the slice the caller passed, beside the span id it already carries. Every joiner then resolves the exact occurrence the row was built from. Adding a field to a public struct is a shape change in a 0.x crate whose only constructor is this workspace, and the round records it as accepted.

Fourth, the three consumers switch to the index. The SVG waterfall draws each row from its own occurrence and takes its window from every occurrence in the trace. The MCP panel detail reports each row's own name, service, and times. The latency finding names the occurrence that holds the largest self time, with that occurrence's own name and service. The finding's critical path clause stays id-based, because a critical path is a chain of ids by definition and the existing stop rule already defines its behavior when an id repeats, so the clause stays truthful at the granularity it claims.

## Scope fences

The flame builder, the sankey staging, and the service graph each walk ids in their own way. The build audits all three under a shared-id input and fixes any place where a named figure or a label crosses between copies. A walk that aggregates by id on purpose, the way a service graph aggregates by service, is not a defect and is recorded as defined behavior where that is the finding. The critical path walk does not change. No panel, dialect, or surface is added, and no behavior changes on a trace whose ids are unique, which the golden corpus pins byte for byte.

## The invariant

INV-89 joins the set in the build. Its draft text reads as follows. Within a trace, spans that share an id are distinct occurrences. Every occurrence renders its own row with its own figures, children attach to the first occurrence of their parent id in canonical order, and any finding or detail that names a span names the occurrence its figures came from. No surface joins a derived row back to a span by id alone.

## The pins

A derive test builds a shared-id pair with distinct services and times and a child naming the shared id, and asserts two rows with each copy's own self time and the child under the first copy. An investigate test gives the two copies different self times and asserts the finding names the service of the copy that holds the larger one. An MCP detail test asserts each row carries its own copy's name and service. An SVG test asserts the waterfall draws both copies. The golden corpus and the scale hashes stand unchanged, because every committed fixture carries unique ids within its traces.
