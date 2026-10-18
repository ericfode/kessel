//! Ordered, reviewed naming data. Title IDs belong to this registry, never to
//! the admitted card pool: an unsupported or absent card can still be named.
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub(crate) struct Registry {
    pub titles: Vec<String>,
}
impl Registry {
    pub fn parse(value: Option<&Value>) -> Result<Option<Self>, String> {
        let Some(value) = value else {
            return Ok(None);
        };
        if value.as_object().is_none_or(|o| o.len() != 3)
            || value["version"] != 1
            || value["sourceHash"] != crate::named_play_contract::REGISTRY_SHA256
        {
            return Err("unsupported playable card title registry provenance".into());
        }
        let titles: Vec<String> = serde_json::from_value(value["titles"].clone())
            .map_err(|_| "playable card title registry requires strings")?;
        let digest = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&titles).map_err(|e| e.to_string())?)
        );
        if titles.len() != crate::named_play_contract::REGISTRY_COUNT
            || digest != "63de4da772e5128ebc536399ff878faa672cc0416616b52098971c3df96fb92d"
        {
            return Err(
                "playable card title registry content/order differs from reviewed data".into(),
            );
        }
        Ok(Some(Self { titles }))
    }
    pub fn fingerprint(&self, legal: Option<&[u16]>) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"kessel-named-play-v1\0");
        hash.update(crate::named_play_contract::REGISTRY_SHA256.as_bytes());
        for id in 0..self.titles.len() as u16 {
            if legal.is_none_or(|ids| ids.contains(&id)) {
                hash.update(id.to_le_bytes());
            }
        }
        hash.finalize().into()
    }
}
