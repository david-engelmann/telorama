# Round 6 design: the registry finish

This record settles how the published crates are
finished. It follows the registry recon of 2026-11-23,
which read the live crates.io and docs.rs pages for
the 0.1.0 release firsthand.

## What the recon found

All five crates published with the repo root readme,
and both registries render it, but neither can resolve
its relative media. crates.io rewrites the image paths
against the crate directory, producing a GitHub raw URL
under crates/telorama-scene that names a file which
does not exist there, and docs.rs rewrites them against
its own host the same way. The hero animation and the
incident picture are broken images on every crate page.

The readme text itself is stale in its current state
section. It says the sankey panel has no renderer yet,
which Round 1 landed. It counts a golden corpus of
twelve scenes, which has grown. It says the crates
stand ready to publish, which they did on 2026-11-22.
It says nothing about the dark theme or the gallery.

The docs.rs builds are healthy. The scene crate's
rustdoc renders its full public surface with module
docs, and the other crates' pages render the readme.
What is missing is any worked example in the API docs
themselves, and any check that the public surface
stays documented.

## The finish

Per-crate readmes. Each crate gains its own README.md
at its root, short and scoped to that crate, and its
Cargo.toml points readme at that file. Media in a
crate readme is an absolute URL to the raw file on
the main branch, the one form both registries serve
as written. The repo root readme keeps its relative
media, which GitHub resolves correctly, and stays
the project's face there.

Root readme currency. The current state section is
rewritten to the finished state. Sankey panels render.
Scenes carry a theme and render in a dark palette
whose contrast is proven. The gallery page exists and
is linked. The crates are published, at the version
this round ships. The golden corpus is counted as it
stands, thirteen scenes with five pinned in both
themes.

Version 0.2.0 for all five crates together. The
changes since 0.1.0 are features, a new panel
renderer, a new scene field with a second palette,
and wider dialect and price coverage, so the bump is
a minor one, not a patch. The workspace dependency
pins move with the crate versions, and Cargo.lock is
regenerated in the packaging pass and committed.

Doc examples. Each library crate gains one worked
example in its front-door documentation, run as a doc
test by the hosted gate. The scene example parses and
validates a small spec. The ingest example parses a
small OTLP export. The svg example renders a scene to
an SVG string. The MCP example answers a tools/list
request through the pure message handler. The CLI
crate is a binary and carries no doc test. Its docs
are its help text and the readme.

A packaging test pins the shape. It reads the five
crate manifests and readmes and asserts the versions
agree, each manifest names its own readme, and no
crate readme carries a relative image or link. It can
fail the day a version is bumped in one crate only or
a relative path slips back in.

Release mechanics are the settled ones. Packaging
runs on a CI runner, the publish runs in the order
scene, ingest, svg, telorama, telorama-mcp with the
stored credential, and each version is confirmed live
before the next is sent. After the release, the
rendered crate pages and the docs.rs pages for 0.2.0
are read firsthand and the media on them verified.

## Fences

No behavior changes anywhere. No new public API
beyond what the doc examples exercise. No change to
any golden, fixture, or figure. The root readme's
relative media stays as it is.
