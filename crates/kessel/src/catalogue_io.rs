//! Load a unified `swu-card-ir/v1` catalogue from a legacy monolith file or a
//! concern-split generated directory (`manifest.json` + shards).
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const IR_SCHEMA: &str = "swu-card-ir/v1";
const MANIFEST_SCHEMA: &str = "swu-card-ir-manifest/v1";
const INPUTS_SCHEMA: &str = "swu-card-ir-inputs/v1";
const DIAGNOSTICS_SCHEMA: &str = "swu-card-diagnostics/v1";
const CARDS_SCHEMA: &str = "swu-card-ir-cards/v1";

pub fn default_generated_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../compiler/generated")
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn is_dir_catalog(path: &Path) -> bool {
    path.is_dir()
        || (path.file_name().and_then(|n| n.to_str()) == Some("manifest.json") && path.is_file())
}

fn catalog_dir(path: &Path) -> PathBuf {
    if path.file_name().and_then(|n| n.to_str()) == Some("manifest.json") {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

/// Assemble the unified catalogue object from a path (file or directory).
pub fn load_catalogue_value(path: &Path) -> Result<Value, String> {
    if !is_dir_catalog(path) {
        let value = read_json(path)?;
        if value["schema"] != IR_SCHEMA || !value["cards"].is_array() {
            return Err(format!("not a {IR_SCHEMA} catalogue: {}", path.display()));
        }
        return Ok(value);
    }
    let dir = catalog_dir(path);
    let manifest = read_json(&dir.join("manifest.json"))?;
    if manifest["schema"] != MANIFEST_SCHEMA {
        return Err(format!("bad catalogue manifest schema in {}", dir.display()));
    }
    let files = &manifest["files"];
    let cards_name = files["cards"].as_str().unwrap_or("cards.json");
    let inputs_name = files["inputs"].as_str().unwrap_or("inputs.json");
    let diagnostics_name = files["diagnostics"].as_str().unwrap_or("diagnostics.json");
    let cards_file = read_json(&dir.join(cards_name))?;
    let inputs_file = read_json(&dir.join(inputs_name))?;
    let diagnostics_file = read_json(&dir.join(diagnostics_name))?;
    if cards_file["schema"] != CARDS_SCHEMA {
        return Err(format!("bad cards shard schema in {}", dir.display()));
    }
    if inputs_file["schema"] != INPUTS_SCHEMA {
        return Err(format!("bad inputs shard schema in {}", dir.display()));
    }
    if diagnostics_file["schema"] != DIAGNOSTICS_SCHEMA {
        return Err(format!("bad diagnostics shard schema in {}", dir.display()));
    }
    let compiler_hash = manifest["compilerHash"].as_str().unwrap_or("");
    for (label, shard) in [
        ("cards", &cards_file),
        ("inputs", &inputs_file),
        ("diagnostics", &diagnostics_file),
    ] {
        if shard["compilerHash"].as_str() != Some(compiler_hash) {
            return Err(format!("{label} shard compilerHash mismatch in {}", dir.display()));
        }
    }
    let mut diagnostics = std::collections::BTreeMap::new();
    for entry in diagnostics_file["cards"]
        .as_array()
        .ok_or("diagnostics shard missing cards")?
    {
        let code = entry["setCode"]
            .as_str()
            .ok_or("diagnostics entry missing setCode")?
            .to_owned();
        diagnostics.insert(code, entry["diagnostics"].clone());
    }
    let cards = cards_file["cards"]
        .as_array()
        .ok_or("cards shard missing cards")?
        .iter()
        .map(|entry| {
            let code = entry["setCode"].as_str().unwrap_or("");
            let mut card = entry.clone();
            card["diagnostics"] = diagnostics
                .get(code)
                .cloned()
                .unwrap_or_else(|| json!([]));
            card
        })
        .collect::<Vec<_>>();
    let mut value=json!({
        "schema": IR_SCHEMA,
        "compilerVersion": manifest["compilerVersion"],
        "typescriptVersion": manifest["typescriptVersion"],
        "sourceTreeHash": manifest["sourceTreeHash"],
        "compilerHash": manifest["compilerHash"],
        "inputs": inputs_file["inputs"],
        "sourceSupport": manifest["sourceSupport"],
        "cards": cards,
        "inventory": manifest["inventory"],
    });
    if let Some(registry)=manifest.get("playableCardTitles") {value["playableCardTitles"]=registry.clone();}
    Ok(value)
}

/// JSON bytes of the assembled catalogue (compact). Suitable for `from_catalogue_bytes`.
pub fn load_catalogue_bytes(path: &Path) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&load_catalogue_value(path)?).map_err(|e| e.to_string())
}

/// Pretty-printed assembled bytes — matches the historical `catalog.json` formatting.
pub fn load_catalogue_pretty_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let mut out = serde_json::to_vec_pretty(&load_catalogue_value(path)?).map_err(|e| e.to_string())?;
    out.push(b'\n');
    Ok(out)
}
