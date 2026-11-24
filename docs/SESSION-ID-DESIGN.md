# Session derivations under a repeated span id

## Why this round exists

Round 16 put the trace derivations under attribution by occurrence and recorded two joins of the same class as named leftovers on the agent-run surface. This record is the charter for closing them. Both recon lanes verified the reports firsthand, and the sweep that came with them found the full extent of the class on the session side.

The shape is the same one Round 16 established as real. A span id repeats within one trace in GenAI and OpenInference exports for the same bridging reasons a Zipkin shared id repeats, and a Claude Code export can carry two tool spans under one id. The session derivations were built as if an id names one span. On the session side the damage is worse than a crossed label, because the session model prices what it walks. A suppressed step takes its tokens and its cost with it, and a doubled walk doubles the headline figures a reader trusts most.

## What the recon found

Three findings in the dialect session walk in the ingest crate, all verified with line evidence. The children map is keyed by parent span id, so the children of two copies of one id merge into one list that either copy is handed. The emit walk tracks visited spans by id, so within a turn the second copy of an id emits no step at all, and a model copy suppressed this way is never priced or counted anywhere. The root loop gives every root copy its own turn with a fresh visited set, so two root copies of one id each walk the merged children and the shared subtree lands in both turns, double-counted in the turn totals and in the session headline that sums over turns. The coverage count in the same walk counts each id once per turn, which understates distinct copies within a turn and disagrees with the double walk across turns.

One finding in the Claude Code fusion, verified with a precision the original report lacked. The blocked map pools every blocked_on_user child under its parent span id, across all copies of that id. The fusion gates each step on its own occurrence's window but reads the wait from the pooled list, so a step takes the union sum of both copies' blocked children as its permission wait. Disjoint copy windows suffice to fire it, because the gate is per occurrence while the pool is per id. The merged coverage count in the same adapter counts merged span ids as strings, so distinct merged occurrences under one id count once, and the pooled path credits one copy's children to the other copy's turn.

Two lesser observations stand recorded with dispositions below. The decision maps in both session adapters key a tool's decision by turn and tool name, so two decisions for one tool name in one turn collapse to the last. The labels the dialect walk derives, the prompt id and the agent id, are the source dialect's own id strings, so two turns can share a prompt id and two agent spans can share an agent id label. The Codex adapter carries no span id join at all and stands clean.

## The defined rule

INV-89 already states the law for this class and the session derivations are simply not under it yet. No new invariant joins the set. The build brings the session derivations under INV-89 and extends its enforcement line to the ingest suites.

First, every span occurrence in a session walk emits its own step exactly once across the whole run, tracked by its position in the caller's slice rather than by its id. A visited set keyed by occurrence replaces the id keyed sets in the emit walk, and the set spans the whole run rather than resetting per turn, so no subtree is walked twice and no copy is suppressed. Termination on parent cycles is preserved, because the occurrence graph is finite and each occurrence is still visited at most once, which is the guard INV-82 has always rested on.

Second, the children of a repeated id are consumed by its adopting copy alone, under the same adopting rule the trace walks use. The adopting copy is the first occurrence in canonical order, or the first reached copy when that occurrence is unreachable when adoption settles. The trace walks and the session walk should share the adopting computation where the shapes allow, as one helper in the scene crate both derivations call, so the rule cannot drift between surfaces. Where the build finds the shapes do not allow sharing, it implements the same semantics locally and records that it did.

Third, the coverage count in the dialect walk counts reached occurrences, so a turn's coverage agrees with the steps the turn actually carries.

Fourth, the Claude Code fusion attributes a blocked child to the tool span occurrence whose window contains the child, and a tool step takes its wait from the children of the one occurrence whose window contains the step. The blocked map keys by the resolved parent occurrence, its position in the caller's slice. A blocked child whose parent id resolves to no containing occurrence stays unattributed rather than joining a namesake. The merged coverage count counts merged occurrences rather than merged id strings.

Fifth, labels stay labels. The prompt id and the agent id remain the source dialect's own strings, recorded as display labels and never used as join keys. The decision maps stand as they are, because a name keyed collapse within one turn is a different class from an id join, no evidence in this round shows a real export carrying two decisions for one tool in one turn, and the round records the observation rather than building on a guess.

## Scope fences

No model shape changes. AgentRun, AgentTurn, and AgentStep keep their fields, and no panel, dialect, or surface is added. The Codex adapter does not change. Session output on inputs whose span ids are unique within a trace stays byte-identical, which the existing session fixtures and goldens pin. The nesting refusal and every other refusal keep their behavior, with the visited re-keying noted where the nesting check reads the visited set.

## The pins

An ingest test builds a GenAI session whose trace carries two copies of one span id with distinct children and asserts both copies emit their own step, the children walk under the adopting copy alone, and the session totals count the shared subtree once. A second test makes both copies roots and asserts two turns whose totals do not double the shared subtree. A Claude Code test builds two tool spans under one id with disjoint windows and a blocked child inside each, and asserts each step's wait is its own occurrence's sum and no other. A coverage assertion rides the first test, that merged and seen counts are occurrence counts. The existing session fixtures stand unchanged.
