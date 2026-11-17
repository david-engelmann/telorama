//! The renderer's own token price table, shared by every dialect
//! that exports tokens but no cost. Prices are US dollars per
//! million tokens. Prices rot; the table date rides on every priced
//! run so a reader can judge the estimate.

use telorama_scene::ModelCall;

/// The date the price table below was compiled.
pub const PRICE_TABLE_AS_OF: &str = "2026-11-16";

/// One model's prices in US dollars per million tokens.
struct ModelPrice {
    input: f64,
    cached_input: f64,
    output: f64,
}

fn price_for(model: &str) -> Option<ModelPrice> {
    match model {
        "gpt-5.1" | "gpt-5.1-codex" => Some(ModelPrice {
            input: 1.25,
            cached_input: 0.125,
            output: 10.0,
        }),
        "gpt-5.1-mini" | "gpt-5.1-codex-mini" => Some(ModelPrice {
            input: 0.25,
            cached_input: 0.025,
            output: 2.0,
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
