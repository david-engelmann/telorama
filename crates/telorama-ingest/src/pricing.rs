//! The renderer's own token price table, shared by every dialect
//! that exports tokens but no cost. Prices are US dollars per
//! million tokens. Prices rot; the table date rides on every priced
//! run so a reader can judge the estimate.

use telorama_scene::ModelCall;

/// The date the price table below was compiled.
pub const PRICE_TABLE_AS_OF: &str = "2026-11-23";

/// One model's prices in US dollars per million tokens.
struct ModelPrice {
    input: f64,
    cached_input: f64,
    output: f64,
}

fn price_for(model: &str) -> Option<ModelPrice> {
    match model {
        "gpt-5.1" | "gpt-5.1-codex" | "gpt-5.1-codex-max" | "gpt-5" | "gpt-5-codex" => {
            Some(ModelPrice {
                input: 1.25,
                cached_input: 0.125,
                output: 10.0,
            })
        }
        "gpt-5.1-mini" | "gpt-5.1-codex-mini" | "gpt-5-mini" => Some(ModelPrice {
            input: 0.25,
            cached_input: 0.025,
            output: 2.0,
        }),
        "gpt-5.2" | "gpt-5.2-codex" | "gpt-5.3-codex" => Some(ModelPrice {
            input: 1.75,
            cached_input: 0.175,
            output: 14.0,
        }),
        "gpt-5-nano" => Some(ModelPrice {
            input: 0.05,
            cached_input: 0.005,
            output: 0.40,
        }),
        "codex-mini-latest" => Some(ModelPrice {
            input: 1.50,
            cached_input: 0.375,
            output: 6.0,
        }),
        "claude-sonnet-5" => Some(ModelPrice {
            input: 2.0,
            cached_input: 0.20,
            output: 10.0,
        }),
        "claude-sonnet-5.5" => Some(ModelPrice {
            input: 2.0,
            cached_input: 0.10,
            output: 10.0,
        }),
        "claude-opus-5.5" => Some(ModelPrice {
            input: 4.0,
            cached_input: 0.20,
            output: 20.0,
        }),
        "claude-opus-5" | "claude-opus-4.5" | "claude-opus-4.6" | "claude-opus-4.7"
        | "claude-opus-4.8" => Some(ModelPrice {
            input: 5.0,
            cached_input: 0.50,
            output: 25.0,
        }),
        "claude-sonnet-4.5" | "claude-sonnet-4.6" => Some(ModelPrice {
            input: 3.0,
            cached_input: 0.30,
            output: 15.0,
        }),
        "claude-haiku-4.5" => Some(ModelPrice {
            input: 1.0,
            cached_input: 0.10,
            output: 5.0,
        }),
        "claude-fable-5.2" | "claude-mythos-5.2" => Some(ModelPrice {
            input: 8.0,
            cached_input: 0.20,
            output: 40.0,
        }),
        _ => None,
    }
}

/// The priced cost of one call, or None when the table has no row for
/// the model.
pub(crate) fn price_call(call: &ModelCall) -> Option<f64> {
    let price = price_for(&call.model)?;
    let per = |tokens: u64, rate: f64| tokens as f64 * rate / 1_000_000.0;
    Some(
        per(call.input_tokens, price.input)
            + per(call.cache_read_tokens, price.cached_input)
            + per(call.output_tokens, price.output),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(model: &str, input: u64, cache_read: u64, output: u64) -> ModelCall {
        ModelCall {
            model: model.to_string(),
            start_unix_nano: 0,
            end_unix_nano: 0,
            ttft_nanos: None,
            attempts: 1,
            is_error: false,
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: cache_read,
            cache_creation_tokens: 0,
            cost_usd: 0.0,
        }
    }

    /// Each figure of a row is pinned on its own: one million
    /// tokens of a single class costs exactly that class's
    /// published figure. Every row in the table is pinned here,
    /// so an edit to any figure, or a swap of two figures within
    /// a row, fails a test that names the model.
    fn row(model: &str, input: f64, cached: f64, output: f64) {
        let cases = [
            (call(model, 1_000_000, 0, 0), input),
            (call(model, 0, 1_000_000, 0), cached),
            (call(model, 0, 0, 1_000_000), output),
        ];
        for (c, expected) in cases {
            let cost = price_call(&c).unwrap_or_else(|| panic!("{model} must have a price row"));
            assert!(
                (cost - expected).abs() < 1e-9,
                "{model}: {cost} != {expected}"
            );
        }
    }

    #[test]
    fn every_table_row_is_pinned() {
        row("gpt-5.1", 1.25, 0.125, 10.0);
        row("gpt-5.1-codex", 1.25, 0.125, 10.0);
        row("gpt-5.1-codex-max", 1.25, 0.125, 10.0);
        row("gpt-5", 1.25, 0.125, 10.0);
        row("gpt-5-codex", 1.25, 0.125, 10.0);
        row("gpt-5.1-mini", 0.25, 0.025, 2.0);
        row("gpt-5.1-codex-mini", 0.25, 0.025, 2.0);
        row("gpt-5-mini", 0.25, 0.025, 2.0);
        row("gpt-5.2", 1.75, 0.175, 14.0);
        row("gpt-5.2-codex", 1.75, 0.175, 14.0);
        row("gpt-5.3-codex", 1.75, 0.175, 14.0);
        row("gpt-5-nano", 0.05, 0.005, 0.40);
        row("codex-mini-latest", 1.50, 0.375, 6.0);
        row("claude-sonnet-5", 2.0, 0.20, 10.0);
        row("claude-sonnet-5.5", 2.0, 0.10, 10.0);
        row("claude-opus-5.5", 4.0, 0.20, 20.0);
        row("claude-opus-5", 5.0, 0.50, 25.0);
        row("claude-opus-4.5", 5.0, 0.50, 25.0);
        row("claude-opus-4.6", 5.0, 0.50, 25.0);
        row("claude-opus-4.7", 5.0, 0.50, 25.0);
        row("claude-opus-4.8", 5.0, 0.50, 25.0);
        row("claude-sonnet-4.5", 3.0, 0.30, 15.0);
        row("claude-sonnet-4.6", 3.0, 0.30, 15.0);
        row("claude-haiku-4.5", 1.0, 0.10, 5.0);
        row("claude-fable-5.2", 8.0, 0.20, 40.0);
        row("claude-mythos-5.2", 8.0, 0.20, 40.0);
    }

    #[test]
    fn a_model_with_no_row_is_unpriced() {
        assert!(price_call(&call("gpt-6.2", 1_000, 0, 1_000)).is_none());
        assert!(price_call(&call("claude-haiku-5.5", 1_000, 0, 1_000)).is_none());
    }

    #[test]
    fn cache_creation_is_never_priced() {
        let mut with_creation = call("gpt-5.2", 2_000, 0, 500);
        with_creation.cache_creation_tokens = 900;
        let plain = call("gpt-5.2", 2_000, 0, 500);
        assert_eq!(price_call(&with_creation), price_call(&plain));
    }
}
