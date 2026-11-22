# INVARIANTS.md

The invariants telorama holds, each traced one to one to the test or
record that enforces it. An invariant without a named enforcement is a
wish, and wishes do not ship. Sections run in invariant number order.
Rules the validator already enforces for panels whose renderers land in
later milestones are marked as validator-side.

## Spec and honesty (telorama-scene, validate.rs)

- INV-1. A scene whose version is not the supported spec version is
  rejected, never silently reinterpreted.
  Enforced by validator_tests unsupported_version_is_rejected.
- INV-2. Duration statistics are percentiles, never averages. A stat
  panel over duration data that asks for an average fails validation
  with the percentile rule named in the message.
  Enforced by validator_tests average_duration_statistic_is_rejected and
  cli_tests validate_rejects_an_average_duration_with_the_honesty_message.
- INV-3. Every data entry states its source kind, its exact query or
  trace id, and its sampling state. Nothing about provenance is implied.
  Enforced by validator_tests source_without_query_or_trace_is_rejected
  and missing_sampling_state_is_rejected.
- INV-4. Sankey panels name an explicit residual bucket, so flow that
  does not balance has somewhere honest to go. Validator-side in M1.
  Enforced by validator_tests sankey_without_residual_is_rejected_and_with_residual_passes.
- INV-5. Flame panels declare a sample type, a sample floor, and a
  sample count, and refuse to validate below the floor. Validator-side
  in M1.
  Enforced by validator_tests flame_below_sample_floor_is_rejected.
- INV-6. Heatmap data declares real histogram bucket edges, strictly
  increasing, on the data source. No interpolated surfaces. Validator-side
  in M1.
  Enforced by validator_tests heatmap_needs_real_bucket_edges.
- INV-7. Data ids are unique, and every panel reads a declared data
  entry.
  Enforced by validator_tests duplicate_data_id_is_rejected and
  panel_reading_undeclared_data_is_rejected.
- INV-8. Narrative items anchor to panels that exist.
  Enforced by validator_tests narrative_anchor_must_exist.
- INV-9. The scene window runs forward. A range whose end is not after
  its start is rejected.
  Enforced by validator_tests backward_window_is_rejected.

## Derivations (telorama-scene, derive.rs)

- INV-10. The critical path is a root to leaf chain of parent to child
  steps, built by the CRISP walk (at each span, follow the child that
  finishes last).
  Enforced by derive_tests critical_path_follows_the_last_finishing_child
  and property_tests critical_path_is_always_a_root_to_leaf_chain over
  1,000 generated traces.
- INV-11. Self time is a span's duration minus the coverage of its
  direct children, and always lands between zero and the duration.
  Enforced by derive_tests self_time_subtracts_children_coverage and
  property_tests self_time_stays_within_duration over 1,000 traces.
- INV-12. Percentiles are nearest-rank, and p50 never exceeds p95 and
  p95 never exceeds p99 on the same data.
  Enforced by derive_tests percentile_is_nearest_rank and property_tests
  percentiles_are_monotone_and_red_ratios_are_bounded.
- INV-13. RED counts a service's server spans when it has them, so one
  served request counts once, and the counting basis is recorded on the
  result. Error ratios stay within zero and one.
  Enforced by derive_tests red_counts_server_spans_when_present and
  property_tests percentiles_are_monotone_and_red_ratios_are_bounded.
- INV-14. Derivations are pure functions of the span set. Input order
  never changes a result.
  Enforced by derive_tests derivations_do_not_depend_on_input_order.

## Ingest (telorama-ingest)

- INV-15. OTLP trace exports parse from a single JSON document or from
  JSON Lines into the one canonical span model, sorted by trace id,
  start, and span id.
  Enforced by ingest_tests json_lines_input_parses_each_document and
  parsing_is_deterministic.
- INV-16. Bad input fails loudly with a specific error. Empty input,
  non-OTLP input, and spans without start times are errors, never empty
  results.
  Enforced by ingest_tests empty_and_garbage_inputs_are_errors_not_empty_results
  and missing_start_time_is_a_specific_error.
- INV-17. Error spans are marked from the OTLP status code, and the
  summary counts them per service.
  Enforced by ingest_tests checkout_fixture_parses_to_nine_spans.

## Rendering (telorama-svg)

- INV-18. The determinism law. The same scene, the same data, and the
  same engine version produce byte-identical SVG, and the order spans
  arrived in never leaks into the render.
  Enforced by golden_tests rendering_is_byte_identical_across_runs_and_input_orders
  and the golden corpus in tests/golden, which gates every change.
- INV-19. Every float in the output passes through one canonical
  formatter, and every timestamp through one canonical time formatter.
  Enforced by golden_tests formatting_is_canonical.
- INV-20. A panel type with no renderer yet fails with a specific error
  naming the panel. Nothing renders silently wrong.
  Enforced by golden_tests unsupported_panel_type_is_a_specific_error.
- INV-21. Service colors come from the Okabe-Ito palette, assigned by
  sorted service name, identical in every panel of a scene. Error spans
  render in vermillion and carry a text marker, so color is never the
  only channel.
  Enforced by golden_tests service_colors_follow_sorted_name_order,
  which asserts the assignment directly, and by the golden corpus
  (golden_tests golden_checkout_scene), which records the rendered
  result byte for byte.

## Schema and CLI (telorama)

- INV-22. The JSON Schema the CLI prints is the shape contract for spec
  version 0.1, generated from the same crate that validates.
  Enforced by cli_tests schema_command_prints_the_scene_schema.
- INV-23. The CLI render path and the library render path produce the
  same scene for the same inputs. Since M5 the surface renders carry
  the findings strip the library goldens do not (INV-57); the scene
  content itself is byte-identical.
  Enforced by cli_tests render_draws_the_golden_scene_with_the_findings_strip.

## Field system (telorama-scene, field.rs)

- INV-24. One formatting language. Every value a scene prints passes
  through the field formatters in telorama-scene; the svg crate's
  formatting functions are re-exports of them, not copies, so an axis,
  a stat, a table cell, and a caption can never disagree.
  Enforced by field_tests format_value_duration_scales_by_magnitude,
  format_value_ratio_is_a_one_decimal_percent, and
  format_value_count_bytes_and_rate, and by the golden corpus.
- INV-25. Threshold steps are strictly increasing in `above` and carry
  #RRGGBB colors, and a value takes the color of the highest step at or
  below it. Threshold `above` values are in the field's own scale, the
  same scale format_value takes.
  Enforced by validator_tests thresholds_must_increase_and_carry_hex_colors
  and field_tests thresholds_pick_the_highest_step_at_or_below_the_value.
- INV-26. A red_row panel with no thresholds of its own renders with
  the standing M1 verdict: an error ratio at or above five percent is
  vermillion. Scenes written before thresholds existed render
  byte-identical.
  Enforced by field_tests error_ratio_default_matches_the_m1_rule and
  the golden corpus.

## Heatmap (telorama-scene derive.rs, telorama-svg)

- INV-27. Heatmap bucketing is the Prometheus `le` rule over the
  declared edges: a span lands in the first bucket whose edge is at or
  above its duration, or in the overflow row above the top edge. Rows
  are the declared buckets and nothing else; no surface is
  interpolated.
  Enforced by derive_tests heatmap_buckets_by_duration_and_start.
- INV-28. The heatmap exemplar is a real span from the rendered set,
  chosen by a stated rule (the longest span in the busiest cell, rows
  scanned in ascending bucket order, columns left to right, span id as
  tie-break), and it is named in the render with its trace id. An
  exemplar is never invented.
  Enforced by derive_tests heatmap_buckets_by_duration_and_start and
  the golden corpus (golden_tests golden_heatmap_scene).
- INV-29. Spans starting outside the scene window are excluded from
  the grid and counted; the render states the count when it is not
  zero. Nothing is silently dropped.
  Enforced by derive_tests heatmap_counts_spans_outside_the_window.

## Flame (telorama-scene derive.rs, telorama-ingest, telorama-svg)

- INV-30. A flame panel over profile data renders only when the
  folded stacks total exactly the sample count the scene declares.
  A scene must not state a count its data does not carry; a mismatch
  is a render error, never a footnote.
  Enforced by golden_tests flame_declared_count_mismatch_is_a_render_error,
  whose scene declares 971 against stacks totalling 970, and by
  ingest_tests folded_stacks_reject_bad_lines for the parse side. The
  matching-count render is recorded by golden_tests golden_flame_scene.
- INV-31. The span fallback is labelled on its face: a flame derived
  from a trace states that it is derived from span self time and is
  not a sampled profile, and its weights are self time in
  nanoseconds. A derived flame is never presented as a profile.
  Enforced by derive_tests flame_from_spans_weights_self_time and
  golden_tests flame_span_fallback_is_labelled_on_its_face, which
  asserts the note text in the rendered SVG.

## Service map (telorama-scene derive.rs, telorama-svg)

- INV-32. A service map edge exists only on span-tree evidence: a
  span whose parent, in the same trace, belongs to another service.
  The child span is counted once, with its duration and error state.
  No edge is inferred from naming, configuration, or co-occurrence.
  Enforced by derive_tests service_graph_edges_come_from_span_parents.
- INV-33. Map layout is a pure function of the span set: layers by
  longest path from a root service (bounded relaxation, so cycles
  terminate), barycenter of predecessors then name within a layer.
  The same spans render the same map, byte for byte.
  Enforced by golden_tests service_map_layout_layers_follow_longest_path,
  which asserts the layer columns and the within-layer order in the
  rendered map, and by the golden corpus (golden_tests
  golden_service_map_scene).

## Agent runs (telorama-scene agent.rs, telorama-ingest claude_code.rs, telorama-svg)

- INV-34. Fusion authority is fixed: tokens and cost come only from
  log events, permission waits and subagent structure only from
  spans. The run's coverage counts (events read, spans read, spans
  merged) print in every agent_run render. Nothing is merged
  silently, and a fact from the weaker source never overrides the
  stronger one.
  Enforced by ingest_tests claude_code_session_fuses_logs_and_spans
  and the golden corpus (golden_tests golden_agent_run_scene).
- INV-35. A subagent exists in the model only where a span carries
  an agent_id. A log step joins a subagent only when the step's
  midpoint falls inside that span's window. Log events alone never
  invent a subagent.
  Enforced by ingest_tests claude_code_session_fuses_logs_and_spans
  (the haiku call nests under agent-7; nothing else moves).

## Codex dialect (telorama-ingest codex.rs, telorama-svg)

- INV-36. In the Codex dialect, tokens come only from log events
  and spans are counted in coverage but never merged: Codex span
  fusion is unproven, so no wait or subagent is read from a Codex
  span. The coverage line on the render states the dialect's
  authority in words.
  Enforced by ingest_tests codex_session_prices_tokens_and_pairs_tools
  (spans_merged stays 0) and the golden corpus (golden_tests
  golden_agent_run_codex_scene).
- INV-37. Codex reports input tokens gross of cache reads. The
  canonical model stores non-cache input only (reported input minus
  cache read), so a cached token is never counted twice.
  Enforced by ingest_tests codex_session_prices_tokens_and_pairs_tools
  (8000 reported with 6000 cached enters the model as 2000).
- INV-38. Codex exports no cost, so the renderer prices tokens from
  a table it owns. The run carries the table date and the split of
  priced and unpriced models, and the render prints the disclosure.
  An unpriced model's tokens count in the run and its cost is
  excluded from every total, never silently zeroed.
  Enforced by ingest_tests codex_session_names_unpriced_models and
  the golden corpus (golden_tests golden_agent_run_codex_scene).

## HTML export (telorama-svg standalone_html)

- INV-39. The standalone HTML export is deterministic and carries
  the byte-identical SVG. It contains no scripts and no external
  references, so the file renders the same offline, anywhere.
  Enforced by golden_tests
  html_export_wraps_the_svg_without_external_references and
  cli_tests render_html_writes_a_standalone_document.

## MCP server and view (telorama-mcp)

- INV-40. The render tool returns the triple output contract:
  concise text for the model and text-only hosts, compact
  structuredContent whose figures come from the same derive
  functions the renderer uses, and the SVG in _meta for the view.
  The SVG never enters model context.
  Enforced by mcp_tests render_scene_returns_the_triple_contract
  (the _meta payload is the byte-identical golden render).
- INV-41. The render tool's definition links the scene view
  resource, and the view is a self-contained MCP Apps document: it
  opens the bridge handshake, renders only the SVG a tool result
  delivers in its _meta, and fetches nothing.
  Enforced by mcp_tests tools_list_links_the_view_and_hides_the_drilldown
  and mcp_tests resources_read_returns_the_app_view.
- INV-42. The panel drill-down tool is app-only: its definition
  carries visibility ["app"], so hosts keep it from the model, and
  its detail is derived by the same functions as the render
  summaries, so the view and the picture never disagree.
  Enforced by mcp_tests tools_list_links_the_view_and_hides_the_drilldown
  and mcp_tests panel_detail_drills_into_one_panel.

## Span dialects (telorama-ingest span_dialects)

- INV-43. Span-dialect runs derive structure and tokens from span
  attributes alone. Coverage for a span dialect reports log events
  zero and spans merged equal to spans seen, so the run never
  claims a fusion it did not perform.
  Enforced by ingest_tests genai_session_maps_spans_to_a_run and
  openinference_session_maps_span_kinds.
- INV-44. The GenAI mapping reads the union of the attribute
  shapes across the convention's generations. The response model
  wins over the request model when a span carries both, and the
  request model alone suffices. Cache read and cache creation
  are each read under both of their attribute names and are
  optional in every generation. The provider attribute is not
  part of the canonical run and is not read.
  Enforced by ingest_tests genai_session_maps_spans_to_a_run.
- INV-45. Span dialects state one session per export. Two distinct
  conversation or session ids in one export are refused rather
  than merged, and an export with no session id falls back to its
  single trace id and refuses a multi-trace export.
  Enforced by ingest_tests genai_session_refuses_two_conversations.

## Incident composition (telorama-scene compose)

- INV-46. Incident composition is a pure function of its inputs.
  The same span set composes the same scene, and the choice rules
  are fixed. The failing service holds the highest error ratio
  among services with errors, ties on more errors then name. The
  exemplar is the longest erroring trace, else the longest trace,
  ties on trace id.
  Enforced by compose_tests
  incident_scene_names_the_failing_service_and_exemplar and
  incident_scene_without_errors_takes_the_slowest_trace.
- INV-47. A composed scene passes the same validator as a hand
  authored scene, and its headline and narrative derive from the
  RED numbers of its own window. An attached run adds exactly one
  panel and one data entry.
  Enforced by compose_tests
  incident_scene_names_the_failing_service_and_exemplar (validator
  round trip), incident_scene_attaches_a_run_as_the_last_panel,
  and golden_tests golden_incident_checkout_scene.

## Plugin packaging (plugin/)

- INV-48. The plugin manifest, the MCP registration, and the
  skill describe the same product. The manifest names telorama
  at the workspace version, the registration starts the
  telorama-mcp binary, and the skill names all seven tools and all
  four dialects.
  Enforced by cli_tests plugin_packaging_is_wellformed.

## Investigation (telorama-scene investigate)

- INV-49. A finding derives only from the derived truths the
  panels print. Its evidence numbers equal the matching
  derivation's numbers over the same inputs, so the strip can
  never quote a figure the panels do not. Each finding also
  carries those numbers as structured evidence beside its
  statement, formatted by the same formatters, so the agent
  reader consumes the same evidence the human reads.
  Enforced by investigate_tests
  investigate_ranks_errors_before_latency_and_matches_red and
  investigate_wait_and_cost_findings_from_a_run, and
  golden_tests golden_incident_findings_scene.
- INV-50. Every finding carries at least one exemplar that
  exists in the input. A candidate whose exemplar cannot be
  found is not emitted.
  Enforced by investigate_tests
  investigate_ranks_errors_before_latency_and_matches_red
  (exemplar membership) and
  investigate_wait_and_cost_findings_from_a_run.
- INV-51. Finding order is a total deterministic order, errors
  before latency before wait before cost, and statements are
  byte-identical for identical input under any input ordering.
  Enforced by investigate_tests
  investigate_is_deterministic_under_input_reordering.
- INV-52. Findings state evidence and never prescribe a remedy.
  Statements are built only from evidence templates over the
  derivations, with no recommendation text anywhere in the
  investigation layer.
  Enforced by investigate_tests (every statement assertion) and
  golden_tests golden_incident_findings_scene.

## Agent state (telorama-ingest state, telorama-svg)

- INV-53. An agent-state entry with an empty action or an empty
  rationale is refused at parse, and an agent_state panel
  rendered without state data fails the render rather than
  drawing an empty panel. The panel reads a data entry whose
  source kind is agent_state; the validator refuses the panel
  bound to any other kind, and the renderer refuses it again,
  so state supplied for another entry never satisfies the panel.
  Enforced by ingest_tests
  agent_state_parse_enforces_the_rationale_rule, validator_tests
  agent_state_panel_must_read_an_agent_state_entry, cli_tests
  render_with_state_refuses_a_misbound_agent_state_panel, and
  golden_tests golden_agent_state_scene.

## Frames (telorama-scene frames)

- INV-54. Frame prefix purity. Frame k is a pure function of
  the session prefix through turn k. Its turns are the prefix
  turns verbatim, its coverage sums the prefix turns' own
  log_events, spans_seen, and spans_merged, its pricing names
  only models the prefix used, and its scene and findings
  speak only of the prefix.
  Enforced by golden_tests
  frames_from_a_truncated_export_equal_the_leading_frames and
  frames_tests frame_pricing_names_only_prefix_models.
- INV-55. Every frame is a complete scene that validates and
  renders standalone from its own prefix, with figures from
  that prefix only.
  Enforced by frames_tests
  every_frame_scene_validates_and_frames_are_deterministic
  and frames_cover_prefixes_in_order, and golden_tests
  frames_from_a_truncated_export_equal_the_leading_frames
  (the standalone render).
- INV-56. The view displays only delivered frames. Stepping
  replaces the whole frame in a container that carries no
  transitions, and payload data never animates.
  Enforced by mcp_tests resources_read_returns_the_app_view
  (the view declares no transitions and carries the stepper).

## Findings on the surfaces (CLI, telorama-mcp)

- INV-57. The CLI and MCP render paths attach findings
  automatically. No flag or argument suppresses them, and the
  tool renders therefore differ from the library goldens by the
  findings strip alone. The MCP session tool attaches the same
  run-only findings to its payload, so the data path carries
  the layer too.
  Enforced by cli_tests render_draws_the_golden_scene_with_the_findings_strip and
  render_agent_run_scene_from_a_session, and mcp_tests
  render_scene_returns_the_triple_contract and
  session_run_tool_prices_a_codex_session.

## Sankey (telorama-scene derive.rs, telorama-svg)

- INV-58. The sankey derivation counts spans, never calls.
  Every span in the panel's data set is exactly one unit of
  flow in exactly one node, staged by its depth in its own
  trace. A span whose parent is in the set links from its
  parent's node. A root span enters at stage zero. A span
  whose parent is named but absent, or whose ancestry cycles,
  enters through the residual node and is counted there. A
  span id that repeats within a trace stages by one canonical
  copy, chosen by the span's own fields alone, and every other
  copy stages by its own parent chain against that staging,
  so the count stays exact and the output stays independent
  of input order.
  Enforced by derive_tests sankey_stages_spans_by_depth,
  sankey_same_service_steps_are_flow,
  sankey_orphans_and_cycles_enter_through_the_residual, and
  sankey_duplicate_span_ids_stage_deterministically.
- INV-59. The accounting is exact: for every node, total
  inflow (parent links plus residual entries) plus the root
  spans staged there equals the node's span count, and the
  residual node's outflow equals the number of spans that
  could not be staged on evidence. Outflow, the count of
  child spans, is a separate figure: fan-out can push it past
  the span count, and the renderer draws each block at the
  greater of the two so no ribbon ever overflows its block.
  Enforced by derive_tests sankey_flow_conserves_on_the_fixtures.
- INV-60. Node order within a stage is value descending then
  service name ascending, and link stacking follows node
  order, so the same data renders byte-identically whatever
  order the spans arrived in.
  Enforced by golden_tests sankey_scene_matches_golden.
- INV-61. The panel prints beneath the diagram how many spans
  entered through the residual bucket, naming the bucket,
  whenever that count is nonzero. When it is zero, no
  residual node is drawn.
  Enforced by golden_tests sankey_scene_matches_golden and
  sankey_orphans_are_named_under_the_diagram.
