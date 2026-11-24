# telorama verdict viewer

A static page that renders one recorded run of the telorama
investigation layer over committed fixture data. It follows the
verdict-viewer treatment David ruled for the fleet's showcase
surfaces, crosscheck first and telorama next, under the design
record in docs/VIEWER-DESIGN.md.

The page renders a record, not a live tool. The record at
data/record.json is assembled from the committed fixtures through
the same library paths the tools use, the incident composition,
the investigation, the frame builder, and the renderer. Nothing
on the page is computed in the browser. The page displays the
delivered SVG and the recorded findings, stepping replaces whole
frames, and payload data never animates. This is demo material
under the Stageshow law. The product is the deterministic engine.

## Build

    npm install
    npm run build

The build is a static export in out/. There is no server.

## The record

Regenerate the record with the freshness gate's update mode, on a
machine that can run the test suite.

    UPDATE_RECORD=1 cargo test -p telorama-mcp --test viewer_record_tests

Run without the variable, the same test asserts the committed
record equals the current tool output byte for byte (INV-91), so
the page cannot drift from the product it shows.
