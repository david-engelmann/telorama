# Watch record: MCP Events, SEP-3415

This record holds the dated state of the MCP Events watch, an
item carried since the applicability audit of 2026-11-23. It is
a watch record, not a design record. Nothing in it authorizes
building anything.

## The state of the proposal, read firsthand on 2026-11-24

SEP-3415, the Events Extension, stands at Final in the
Extensions Track. It was created on 2026-10-05 and merged on
2026-11-19. The extension is identified as
io.modelcontextprotocol/events and is defined for protocol
revision 2026-07-28 and later. Its normative text is published
in the official extension repository, modelcontextprotocol/
ext-events, under specification/beta/events, at the Beta stage
of the extension lifecycle.

The mechanism, in brief. A server lists its event types with
events/list, each type carrying a name, an inputSchema for
subscription arguments, a payloadSchema, and the delivery modes
it supports. A client subscribes with a name and arguments and
receives occurrences carrying an event id, a timestamp, a
payload, and a cursor that lets it resume after a disconnect.
Delivery runs three ways, and no mode is mandatory. Poll is
request and response through events/poll. Push holds a
long-lived events/stream request open. Webhook registers an
https callback through events/subscribe, and the server posts
signed events to it under a negotiated TTL.

The premise matters more than the mechanics. Events exists for
servers that front an upstream system where things happen when
nobody is asking, the examples the SEP itself gives being a
Slack message, a GitHub push, and a PagerDuty incident. The
client support picture, read on 2026-11-22 from the official
support matrix and reported in the MCP Bits newsletter of
2026-11-24, is two clients with any Events entry at all, one
full and one partial and webhook-only. Support in the official
SDKs is still to come, and the extension schema and
conformance tests remain open work.

## The disposition for telorama

The watch continues, and its basis has changed. In November
2026 the item was a draft that might never land. It has now
landed as a Final SEP with a Beta extension, so the question is
no longer whether Events will exist. The question is whether
telorama ever has anything for it to carry, and today it does
not. telorama-mcp fronts no upstream system. Nothing happens
inside telorama between calls, no feed arrives, and no source
exists that an events/list response could honestly name. A
server that listed zero event types would be theatre, and a
server that invented an event source to fill the list would be
worse. The investigation and rendering tools take data the
client already has and answer, which is the request-driven
shape the SEP's own motivation section describes Events as
complementing, not replacing.

Two named triggers reopen this item as a design question
rather than a watch. The first is telorama gaining an upstream
source of its own, for example a live telemetry feed it fronts
for investigation, at which point findings arriving on that
feed become a genuine event type. The second is the extension
leaving Beta with SDK support in a stack telorama builds on,
paired with a designed watcher mode that consumes events as a
client. Either trigger earns a design record of its own. Until
one fires, the re-check cadence is the next MCP protocol
revision or the extension leaving Beta, whichever the seat
notices first.
