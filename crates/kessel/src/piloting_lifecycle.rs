//! Whole-card rejection until native Piloting mode, alternate payment and host
//! selection are implemented together. Source IR and dormant attachment helpers
//! remain available for inspection; attached-state support is not play support.
//! Inspect raw values before CardInfo/normalization can discard native metadata.
use serde_json::Value;
pub(crate) const REASON: &str =
    "native Piloting play lifecycle (mode, alternate cost and host selection) is not implemented";
const MAX_DEPTH: usize = 128;
const MAX_NODES: usize = 65_536;
const BUDGET_REASON: &str = "native lifecycle inspection budget exceeded";
fn metadata(v: &Value) -> bool {
    v.get("piloting").is_some_and(|v| !v.is_null())
        || v.get("pilotTag").is_some_and(|v| !v.is_null())
        || ["pilotText", "pilotBox", "pilotingBox"].iter().any(|k| {
            v.get(*k)
                .is_some_and(|v| !v.is_null() && v.as_str() != Some(""))
        })
}
fn normalized_flags(v: &Value) -> bool {
    ["pilotingAbility", "pilotUnitAbility"].iter().any(|k| {
        v.get(*k)
            .is_some_and(|v| !v.is_null() && v.as_bool() != Some(false))
    })
}
/// Bounded raw-Value inspection also protects callers that bypass JSON parsing.
/// Account for queued children before allocating them, not just visited nodes.
fn inspect(root: &Value) -> Result<(), String> {
    let mut remaining = vec![(root, 0usize, false)];
    let mut scheduled = 1usize;
    while let Some((v, depth, keyword_scope)) = remaining.pop() {
        if depth > MAX_DEPTH {
            return Err(BUDGET_REASON.into());
        }
        if normalized_flags(v)
            || (keyword_scope
                && (v
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case("piloting"))
                    || v.as_object()
                        .is_some_and(|o| o.keys().any(|k| k.eq_ignore_ascii_case("piloting")))))
            || matches!(
                v["kind"].as_str(),
                Some(
                    "pilotingAttachedKeyword"
                        | "pilotingAttachedModifier"
                        | "pilotingRegistration"
                        | "pilotTag"
                )
            )
            || v["kind"] == "attachedAbilityRegistration" && v["registrationMode"] == "piloting"
        {
            return Err(REASON.into());
        }
        let children = match v {
            Value::Object(o) => o.len(),
            Value::Array(a) => a.len(),
            _ => 0,
        };
        scheduled = scheduled
            .checked_add(children)
            .filter(|n| *n <= MAX_NODES)
            .ok_or_else(|| BUDGET_REASON.to_string())?;
        if children > 0 && depth == MAX_DEPTH {
            return Err(BUDGET_REASON.into());
        }
        match v {
            Value::Object(o) => remaining.extend(o.iter().map(|(k, v)| {
                (
                    v,
                    depth + 1,
                    keyword_scope || k == "keywords" || k == "keywordValues",
                )
            })),
            Value::Array(a) => remaining.extend(a.iter().map(|v| (v, depth + 1, keyword_scope))),
            _ => {}
        }
    }
    Ok(())
}
/// Pool cards have no source envelope; their raw metadata and injected native
/// wrappers must be checked before the permissive CardInfo projection.
pub(crate) fn reject_card(card: &Value) -> Result<(), String> {
    if metadata(card) {
        Err(REASON.into())
    } else {
        inspect(card)
    }
}
/// Catalogue rejection is per entry so a retained unsupported Pilot never
/// prevents loading an unrelated selected pool from the same full catalogue.
pub(crate) fn reject_entry(entry: &Value) -> Result<(), String> {
    if metadata(entry) || metadata(&entry["card"]) {
        Err(REASON.into())
    } else {
        inspect(entry)
    }
}

/// An alias cannot launder a rejected native lifecycle by dropping metadata.
/// Follow existing aliasOf identities with bounded work and reject cycles.
pub(crate) fn reject_catalogue_entry(entry: &Value, catalogue: &Value) -> Result<(), String> {
    reject_entry(entry)?;
    let mut current = entry;
    let mut visited = std::collections::BTreeSet::new();
    while let Some(alias) = current["aliasOf"].as_str() {
        if visited.len() >= MAX_DEPTH {
            return Err(BUDGET_REASON.into());
        }
        if !visited.insert(alias) {
            return Err("cyclic native lifecycle alias".into());
        }
        let Some(target) = catalogue["cards"]
            .as_array()
            .and_then(|cards| cards.iter().find(|e| e["setCode"] == alias))
        else {
            break;
        };
        reject_entry(target)?;
        current = target;
    }
    Ok(())
}
