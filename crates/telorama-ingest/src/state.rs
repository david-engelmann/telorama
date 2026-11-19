//! Agent-state parsing. The working agent reports its own state as a
//! JSON list of entries; this parser is where the rationale rule is
//! enforced. An entry with an empty agent, action, or rationale is
//! refused with a named error, because the scene will not render the
//! lazy version of an agent's work.

use crate::IngestError;
use telorama_scene::AgentStateEntry;

fn fail(message: impl Into<String>) -> IngestError {
    IngestError {
        message: message.into(),
    }
}

/// Parse an agent-state document (a JSON array of entries) and hold
/// every entry to the reporting rules.
pub fn parse_agent_state(text: &str) -> Result<Vec<AgentStateEntry>, IngestError> {
    let entries: Vec<AgentStateEntry> = serde_json::from_str(text)
        .map_err(|e| fail(format!("agent state is not a JSON array of entries: {e}")))?;
    if entries.is_empty() {
        return Err(fail("agent state has no entries"));
    }
    for (index, entry) in entries.iter().enumerate() {
        for (field, value) in [
            ("agent", &entry.agent),
            ("doing", &entry.doing),
            ("why", &entry.why),
        ] {
            if value.trim().is_empty() {
                return Err(fail(format!(
                    "agent state entry {} has an empty {field}: the report must say who is acting, what it is doing, and why",
                    index + 1
                )));
            }
        }
    }
    Ok(entries)
}
