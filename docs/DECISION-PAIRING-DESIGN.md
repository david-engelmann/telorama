# Decision pairing and the fusion tie-break

## Why this round exists

Round 17 recorded two observations and built on neither. The decision maps in both session adapters key a tool's permission decision by turn and tool name, and a second decision for one name in one turn silently overwrites the first. The Claude Code fusion's blocked-child resolution, smallest containing window, was defined by a code comment and exercised by no test. This record is the charter for closing both, on the evidence two recon lanes gathered firsthand.

## What the recon found

The decision crossing is real, with a narrower trigger than Round 17 recorded. Both adapters build their steps in one pass over the turn's events in time order, and a tool step reads the decision map at its own point in that pass. A step therefore takes the latest decision emitted at or before its own result event, never a later one. In a strictly sequential turn, decision, result, decision, result for one tool name, each result reads its own decision and nothing crosses. The crossing needs the batched interleaving that concurrent tool calls produce, where both decisions are logged before either result lands. The stream decision accept, decision reject, result, result labels the first result reject, which is the second invocation's label, and the accept is lost. The mirror residue exists too, where a result emitted before any decision for its name reads no decision even when its own is logged later in the turn.

No invocation-scoped identifier exists anywhere in either dialect's events. A decision event and a result event each carry the session id, the turn id, and the tool name, and nothing else a join could use. The only signals are turn, tool name, and event order, which is why the map keys on name at all. The committed corpus never exercises the shape. No fixture turn carries two decisions for one tool name, and none carries two tool steps of one name. Whether real exports emit the batched interleaving is not decidable from the code or the fixtures, and the round proceeds on the same footing as the earlier occurrence rounds, that a silent crossing of a trust-sensitive label is worth a defined rule before a field report forces one. The crossed value is a label only, and no figure in the run is computed from it, but the label it can invert is the permission decision itself, an accept landing on the invocation the user rejected.

The tie-break needs no code at all. The resolution already implements a precise and deterministic rule. A blocked child resolves among the tool occurrences whose id matches its parent and whose window contains it at both ends inclusively, to the one minimizing duration and then start, with exact ties falling to the earliest occurrence in the input slice. Nothing downstream re-derives the choice, so a pin at ingest pins it everywhere. What the rule lacked is a test whose shape can tell it apart from its rivals.

## The defined rules

First, decisions pair to results by occurrence within a turn and tool name, as a queue. A decision event appends its value to the queue for its turn and tool name. A result event takes the oldest unconsumed decision from its queue as its label, and a result that finds its queue empty takes no decision, never a later one and never another invocation's leftover. Decisions left over after the last result label nothing, as they do today. The rule rides the idiom the Codex adapter already uses to join its time-to-first-token events to model calls, a queue on the same principle, so both adapters change by one type and two call sites each. On any input where each turn and tool name carries at most one decision, or where decisions and results strictly interleave, the queue returns exactly what the map returns today, and every committed fixture stays byte-identical.

The residue is stated plainly. If an export logs results in a different completion order than the decisions were emitted, pairing by emission order can still mismatch a label. Emission order is the only order the export carries, since no invocation identifier exists, so the rule is a defined approximation of the same standing as the other occurrence dispositions. It strictly dominates the current rule, which mislabels even when the counts match and only the interleaving is batched.

The pairing joins the invariant set as INV-90, stated narrowly over the two adapters' decision events. The tie-break stands under INV-89 as it is, needing only its pin.

Second, the tie-break pin takes the recon's nested shape. Two tool spans share one id with nested windows, the larger first in the export, and one blocked child sits inside both. The child resolves to the smaller occurrence, so the step only the larger occurrence contains keeps a zero wait while the step inside both takes the child's full duration. That pair of assertions kills the largest-window, earliest-start, first-in-input-order, and pooled-by-id rules, and the nonzero wait kills dropping the child as ambiguous.

## Recorded and standing

Three adjacent choices in the same fusion block stand as defined behavior, recorded here rather than built on. A step contained in two tool occurrences takes the larger of the two per-occurrence sums as its wait, never their total. Children enter merged coverage only when their occurrence's sum actually sets a step's wait, so the count is order-sensitive by construction. The wait fusion contains at both ends inclusively while the subagent fusion captures by midpoint with a half-open end, two containment conventions in one pass, each deterministic and each standing since before this round. The silent drop of a blocked child no occurrence contains is already recorded as Round 17's accepted deviation and does not reopen.

## Scope fences

No model shape changes, no new event kinds parsed, and no surface changes. Both adapters change symmetrically. Session output on the committed corpus stays byte-identical, which the standing fixtures and pins prove rather than assume.

## The pins

One decision pin per adapter builds the batched interleaving, decision accept, decision reject, result, result for one tool name in one turn, and asserts the steps read accept then reject in result order. The tie-break pin builds the nested shape above and asserts the two waits and the merged count. The standing decision assertions and the decision-only-prompt test stand unchanged.

## Amendment from the build (2026-11-23)

The neutrality sentence above says the queue returns exactly what the map returns wherever each turn and tool name carries at most one decision. The build's red team found one sub-shape inside that wording where it does not hold. With one decision followed by two results for the same name in one turn, the old map's read did not consume, so both results carried the decision. The queue labels the first result and the second carries none, which is exactly what INV-90 defines. The fence therefore holds for count-matched streams, for strictly interleaved streams, and for the whole committed corpus, and the one-decision-two-results shape stands as a deliberate difference, recorded here the way Round 17 recorded its deviation. A single permission decision labeling two distinct invocations is the crossing class in milder form, and consumption is the principled reading.
