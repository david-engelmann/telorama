# INVARIANTS.md

The invariants telorama holds, each traced one to one to the test or
record that enforces it. An invariant without a named enforcement is a
wish, and wishes do not ship. Milestone 1 invariants are listed first.
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

## Schema and CLI (telorama)

- INV-22. The JSON Schema the CLI prints is the shape contract for spec
  version 0.1, generated from the same crate that validates.
  Enforced by cli_tests schema_command_prints_the_scene_schema.
- INV-23. The CLI render path and the library render path produce the
  same bytes for the same inputs.
  Enforced by cli_tests render_matches_the_golden_file.
