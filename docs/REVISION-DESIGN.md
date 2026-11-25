# Design record: the protocol revision move, Round 51

Settled 2026-11-25 before the build, the same discipline
as every earlier design record in this folder. Round 49
read the MCP ground again firsthand and found the core
protocol standing two revisions past the one telorama-mcp
negotiates. This record settles what the server does
about that gap. It names the target, the negotiation
shape on the transport the server actually serves, the
way the MCP Apps surface composes with the new shape,
the fences the build may not cross, the tests that prove
the result, and the invariant that governs it
afterwards. Every protocol fact below was read firsthand
from the specification's own pages on 2026-11-25, and
the page is named beside the fact it carries. Every code
fact was read in the tree the same day. Where the
evidence ran out, this record says so and builds nothing
on the gap.

## The ground the design stands on

The server today is one pure function. In
crates/telorama-mcp/src/lib.rs, `handle_message` takes a
JSON-RPC line and returns a response line, and the
binary in crates/telorama-mcp/src/main.rs does little
but move lines between stdio and that function,
answering a line that is not valid UTF-8 with a parse
error of its own under INV-88. The initialize branch of
the dispatch answers with the constant
`PROTOCOL_VERSION`, which names revision 2025-06-18, and
with the tools and resources capabilities and the server
name and crate version. Nothing is stored from the
exchange. The handler reads no field of the initialize
params, keeps no negotiated version, and answers every
later message from the message alone. A notification,
any message without an id, earns no reply, which is how
notifications/initialized is already handled. The rest
of the dispatch is ping, tools/list, tools/call,
resources/list, and resources/read, with method not
found for everything else.

The Apps surface sits in the same file and is smaller
than it looks from outside. The server lists and serves
one resource, the view document at `ui://telorama/scene`
with the Apps mime type, and it links that resource from
the render and frame tools through the ui object in
their `_meta`, with the drill-down tool marked app-only
in the same place. The view document itself speaks the
app bridge, and it speaks it to the host, not to this
server. Its script posts the ui/initialize request to
the parent window with the Apps protocol version
2026-01-26, waits for the host to answer, sends the
initialized notification, and then renders what the tool
result notification delivers. No ui message ever reaches
`handle_message`.

The specification's ground moved in one direction. The
2026-07-28 revision is the current one, and its
Versioning and Compatibility page, updated 2026-11-24,
divides implementations into modern, legacy, and
dual-era. Modern means revision 2026-07-28 and later,
where version, identity, and capabilities travel as
per-request metadata. Legacy means 2025-11-25 and
earlier, where an initialize handshake opens the work.
The same page retires the handshake and the session
header, and its companion Overview page, updated
2026-11-18, states the statelessness rule the revision
runs on, that a server must not rely on any earlier
request over the same connection and must take version
and capabilities from each request itself. The stdio
binding in the same revision, updated 2026-11-18, keeps
the framing telorama already speaks, one newline
delimited JSON-RPC message per line, with all request
metadata inline in the message body.

## The target

The pick is dual-era. The server keeps answering the
legacy handshake at 2025-06-18 exactly as it does today,
and it learns to answer the modern revision 2026-07-28
beside it, on the same process and through the same pure
function.

Three facts decide it. The first is what the backward
compatibility guidance actually promises, which is less
than its name suggests. Read on the Versioning and
Compatibility page, the guidance is written for clients
and for servers that choose both eras. It defines how a
client detects a server's era, on stdio by probing with
server/discover and falling back on any answer that is
not a recognised modern error. It says a server that
wishes to serve both populations may implement both
behaviours, and that the two may run concurrently on one
process. To a legacy client facing a modern-only server
it promises nothing at all. The page's own matrix scores
that pairing as fails, and it notes that legacy clients
have no fall-forward mechanism, so the one courtesy it
asks of a modern-only server is an error to an
initialize request that names the versions it does
support. A modern-only move would therefore cut off
every host that speaks to telorama today, and telorama's
hosts are not telorama's to upgrade. The server ships
inside a Claude Code plugin and inside hosts their
owners run on their own cadence.

The second fact is the price of dual service here, which
the tree makes unusually low. Dual-era selection in a
stateful server means tracking which era each session
opened in. This server tracks nothing, so selection
collapses to a reading of the message in front of it, in
one function, with the legacy path being the code
exactly as it stands. The third fact is direction of
travel. Modern clients are no longer hypothetical, the
current revision is the one new hosts implement, and a
server that cannot answer a discover probe reads to such
a host as a legacy server it may choose not to drive.
Dual-era is the only target that serves the installed
base and the arriving one with the same binary.

## The negotiation shape on stdio

Era is chosen per message, from the message's own shape,
at the head of `handle_message`. An initialize request
takes the legacy path and is answered as it is today. A
request whose params carry the modern envelope takes
the modern path. Any other request takes the legacy
path, which is the behaviour the server has always had,
including its standing leniency that a bare tools/list
or tools/call is answered without any handshake at all.
The Versioning and Compatibility page scopes legacy
semantics to the stdio process, and a server with no
memory satisfies that scoping trivially, since every
legacy message is answered as though the process held
exactly the state the message implies, which is none.

The modern envelope is the pair of required keys the
Overview page names in its per-request protocol fields.
The protocolVersion key must be present in the request
`_meta` and must name a modern revision this server
implements, which is 2026-07-28 alone. The
clientCapabilities key must be present beside it. A
request that carries the envelope with a required field
missing is malformed, and the Overview page fixes the
answer as error -32602. A request whose declared version
is anything else, an unknown revision, a future one, or
a legacy revision claimed through `_meta` rather than
opened with initialize, earns the modern version error,
code -32022, whose data names the requested version and
the supported set. Following the worked example on the
Versioning and Compatibility page, the supported set in
that error names both revisions this server speaks,
2026-07-28 and 2025-06-18, because the error is defined
as listing the versions the server supports and the
dual-era server does support both. The clientInfo key is
read for nothing and required for nothing, since the
server keys no behaviour off client identity. The
capabilities object is likewise read only for presence.
The server relies on no capability the client has not
declared, as the Overview page requires, because it
relies on no client capability at all.

Discovery is implemented, because the modern revision
leaves the server no choice. The Discovery page, updated
2026-11-09, states that servers must implement
server/discover and shows its shape. The request carries
no params beyond the envelope, and it follows the same
envelope rule as every modern request, since the
Overview page makes the required fields a condition on
requests as such. The result carries the resultType
complete, a supportedVersions list, the same capabilities
object the legacy initialize answer carries, and the
server identity in the result `_meta` under the
serverInfo key. The supportedVersions list names
2026-07-28 alone. Discovery advertises the versions a
modern client may choose for the modern lifecycle, and a
client that chose 2025-06-18 from that list and then
spoke it through `_meta` would meet the version error
naming it as supported, which would be an incoherent
advertisement. The legacy era is discovered the way it
always has been, by the initialize answer itself.

Every modern result carries the modern envelope on the
way out. The Overview page requires a resultType field
on every result, so each modern result gains resultType
complete at its head. The same page asks servers to
identify themselves in every result `_meta` under the
serverInfo key, and this design takes that request, so
each modern result also gains that stamp, merged into
whatever `_meta` the result already carries, which for
tool results is the telorama payload keys. Four of this
server's results stand on the revision's cacheable list,
and the Caching page in the same specification states
that servers must include the two cache hints on
complete results from them, naming server/discover,
tools/list, resources/list, and resources/read among the
operations it covers. SEP-2549, which stands Final on
the SEP index, defines the fields. The values this
design settles are a ttlMs of 3600000, the hour the
Discovery page's own example prints, and a cacheScope of
public, for all four. The reasoning is that every one of
those payloads is a constant of the compiled binary,
identical for every caller and unable to change during
the life of the process, which is exactly the case the
Caching page describes the public scope as fitting.

Two smaller consequences of the revision are settled
here rather than left to the build. The ping method was
removed in the modern revision, so the modern path
answers ping with method not found, -32601, while the
legacy path keeps today's empty result. And an
initialize request that arrives wearing the modern
envelope is not a modern method at all, so the modern
path answers it -32601 as well. The legacy path is where
initialize lives.

No session state changes hands in any of this, because
there is none to change. The current code stores nothing
from initialize, and the modern path stores nothing from
the envelope. The `PROTOCOL_VERSION` constant in lib.rs
becomes two named constants, one per era, and the
supported set the version error prints is built from
those two names and nothing else.

## How the Apps surface composes

The Apps handshake sits beside the core negotiation, not
inside it, and the tree already shows why. The
ui/initialize exchange runs between the view and the
host over the parent window, at the Apps protocol
version 2026-01-26 that SEP-1865 fixed and Round 49
confirmed still stands as the stable Apps revision. The
server is not a party to it. The server's whole Apps
obligation is the resource, its mime type, and the ui
links and visibility marks in the tool definitions, and
every one of those is era blind, served identically
whether the asking request opened with a handshake or
carried an envelope.

What the core revision changes around Apps is only how
the client declares the extension. The Versioning and
Compatibility page covers extension negotiation in its
own section and gives the Apps extension, identified as
`io.modelcontextprotocol/ui`, as its worked example of a
client advertising an extension in its capabilities. In
the legacy era that advertisement rides the initialize
params. In the modern era it rides the
clientCapabilities of each request. telorama reads
neither today, in keeping with the stable Apps position
that a host without Apps support simply treats the tool
as a standard tool, so the move changes nothing in how
the server decides to serve the view. The view document,
`VIEW_HTML` in lib.rs, is not touched by this design at
all. Its bridge version stays 2026-01-26, because that
version belongs to the Apps specification and moves only
when that specification moves.

## What does not change

Each item here is a fence the build may not cross, not a
hope.

The seven tools keep their names, their input schemas,
and their result contents. `tool_definitions` in lib.rs
is not edited, and the catalogue a client reads is the
same in both eras.

The triple contract holds. A render still returns its
concise text, its structuredContent, and its SVG under
the telorama keys in `_meta`, with the frame stream
under its own key. The modern era adds envelope fields
around that contract and changes nothing inside it.

The transport stays stdio, and the loop in main.rs is
not edited. There is no Streamable HTTP work in this
move, and the routing headers of SEP-2243 belong to that
transport and stay out of scope.

JSON-RPC 2.0 stays the wire grammar in both eras, as the
Overview page itself keeps it.

The view document is not edited, for the reasons the
Apps section gives.

Every other crate is untouched. The whole build lives in
crates/telorama-mcp, in lib.rs and its tests, plus the
invariant in docs/INVARIANTS.md and the record in
DECISIONS.md. The command line tool, which speaks no
MCP, does not change.

## The test and conformance plan

The plan starts from a property worth stating plainly.
No existing test changes its expectations. The tests in
crates/telorama-mcp/tests/mcp_tests.rs that pin the
negotiation today,
initialize_returns_server_info_and_capabilities,
notifications_get_no_reply, and
unknown_method_is_a_protocol_error, all exercise the
legacy or bare paths, and those paths are fenced to stay
byte for byte what they are. The same holds for the
stdio test that opens with a bare initialize against the
spawned binary, and for the library example in the
lib.rs doc comment, which calls tools/list with no
handshake and stays true on the legacy path. A build
that needs to edit an existing expectation has crossed a
fence, and that is a finding against the build, not an
update to make.

The new behaviour is pinned by new tests in the same
file, written in the file's own style of inline JSON
requests through `handle_message`, so no new fixture
files appear. The names below are the names INV-94 will
cite, and each test lands with the slice that builds its
behaviour.

discover_advertises_the_modern_revision sends
server/discover with a well formed envelope and pins the
whole result, the resultType, the single supported
version, the capabilities, the serverInfo stamp, and the
cache hints.

modern_tools_list_carries_the_envelope sends tools/list
with the envelope and pins resultType, the stamp, the
two cache hints, and a tools array equal to the legacy
answer's.

modern_request_without_capabilities_is_invalid_params
sends a modern request missing clientCapabilities and
pins error -32602.

modern_request_with_an_unknown_version_names_the_supported_set
declares a revision the server does not implement and
pins error -32022 with the requested version and both
supported revisions in its data.

modern_ping_is_method_not_found pins the removal of
ping on the modern path, against the legacy traffic that
keeps it.

legacy_and_modern_requests_interleave_on_one_handler
runs a legacy initialize, a modern tools/call, and a
bare tools/call through one handler in sequence and pins
each answer in its own era's shape, which is the test
that proves era selection is per message and no state
bleeds between eras.

modern_render_keeps_the_triple_contract calls
render_scene in both eras over the checkout fixture and
pins the text, the structuredContent, and the SVG as
equal across eras, with only the envelope fields
differing.

Conformance beyond the unit tests rests on the hosted
gate the repo already runs, the full workspace suite on
the exact head. The existing stdio binary test already
proves the legacy era through the real transport, and
the interleave test proves both eras through the handler
the binary delegates to, so no new harness is built.

## The invariant

The build adds one invariant to the MCP server and view
section of docs/INVARIANTS.md, in the house form.

- INV-94. The MCP server is dual-era. It answers the
  legacy initialize handshake at revision 2025-06-18
  exactly as it always has, and it answers any request
  whose _meta declares revision 2026-07-28 under that
  revision's rules, with server/discover implemented,
  the supported set named in the version error, and
  resultType and the serverInfo stamp carried on every
  modern result. Era is chosen per message from the
  message's own shape, and the server keeps no state
  between messages in either era.
  Enforced by mcp_tests
  discover_advertises_the_modern_revision, mcp_tests
  modern_tools_list_carries_the_envelope, mcp_tests
  modern_request_without_capabilities_is_invalid_params,
  mcp_tests
  modern_request_with_an_unknown_version_names_the_supported_set,
  mcp_tests modern_ping_is_method_not_found, mcp_tests
  legacy_and_modern_requests_interleave_on_one_handler,
  and mcp_tests modern_render_keeps_the_triple_contract.

## Sequencing and size

The build lands in three slices, each independently
gated by the full workspace suite and each leaving the
server in a state that could ship.

The first slice is the modern path for the methods that
already exist. Era classification lands at the head of
`handle_message`, the two revision constants replace the
one, envelope validation and its two errors land, and
modern results gain resultType and the serverInfo stamp.
Its tests are the interleave test, the two error tests,
the ping test, the envelope test without its cache hint
assertions, and the render contract test. After this
slice a modern client can drive every tool the server
has, in fully modern shape.

The second slice is discovery and the cache hints.
server/discover lands with its test, and the four
cacheable results gain ttlMs and cacheScope, with the
hint assertions joining the two tests that pin those
results. After this slice the era probe a modern host
opens with is answered, which is the slice that makes
the first slice discoverable.

The third slice is the record. INV-94 enters
docs/INVARIANTS.md, the crate and repo documentation is
checked for revision claims and corrected where it names
one, and the DECISIONS entry for the round is written.
No code moves in this slice.

The size is honest and small. The change concentrates in
the head of one function and in one result wrapping
seam, because the tool implementations below the
dispatch are untouched and the legacy path is the code
as it stands. The risk is not surface area. The risk is
envelope exactness against the specification's worked
examples, which is why the new tests pin whole results
rather than sampled fields.

## What the move does not buy

It does not buy an events surface. SEP-3415 defines
Events for revision 2026-07-28 and later, so this move
removes one obstacle the Round 49 re-check named, and
that is all it does. The extension is still in Beta,
still without support in the official Rust SDK, and
still behind the two named triggers in
docs/EVENTS-WATCH.md, neither of which this move fires.
No events method is implemented, no watcher mode is
designed, and the watch record stands as written.

It does not buy server initiated anything. The modern
revision replaces server requests with the Multi
Round-Trip pattern of SEP-2322, and telorama never sends
server requests, so there is nothing to convert and no
input_required result is introduced.

It does not buy a second transport, subscriptions, or a
hosting story. The server remains a stdio subprocess
that answers questions about data the client already
holds, which is the shape the dossier settled and this
record does not reopen.

## Open questions

Settled by the evidence. Whether the legacy revision
breaks under the new one (it does not, since a Final
revision is frozen and the compatibility guidance is
written around its survival). Whether the stdio
transport survives the revision (it does, with its
framing unchanged in the revision's own binding).
Whether the server holds session state that a stateless
revision would strand (it holds none, by inspection of
lib.rs). Whether server/discover is optional for the
server (it is not, and the Discovery page says so in
terms).

Settled by this design's pick. Dual-era over modern-only
(the target section). The supportedVersions list in
discovery naming the modern revision alone while the
version error names both (the negotiation section). The
cache hint values (the negotiation section). Ping
removed on the modern path and kept on the legacy path.
Legacy results staying byte for byte as they are, with
the envelope carried only by modern results.

Genuinely open for the owner. Three items. The first is
the Apps draft. Round 49 recorded that a draft revision
of the Apps specification is in development, and nothing
in the stable text settles whether the draft changes the
app bridge this view speaks. If it lands before this
build does, the Apps section of this record needs a
re-read against it. The second is the server side
advertisement for the ui extension in discovery. The
core page shows extensions advertised in capabilities,
and the stable Apps text makes the client the declaring
party, and this design ships no server side ui entry in
the discover capabilities. Whether the Apps
specification expects one, and with what settings
object, is to be read from that specification's own text
before the discovery slice lands, and the discovery test
will pin whatever it settles. The third is retirement.
No date is set for dropping the legacy era, and none is
implied by the specification, whose Final revisions are
frozen rather than deprecated. Retirement is an owner
call to be made from host movement, and this record
deliberately leaves it unmade.
