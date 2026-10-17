//! Pure contracts for native Credit identities and the pinned Credit cost stage.
//!
//! This module does not admit cards or execute payments. The engine must wire
//! every resource-cost entry point before enabling native Credit producers.
use serde_json::{json, Value};

pub(crate) const CAPACITY: usize = 256;

/// Physical BaseZone insertion order, independent of ordinary zone capacity.
/// Zero is never a physical ID. Callers retain ownership and presence on cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreditList {
    ids: [u8; CAPACITY],
    len: u16,
}
impl Default for CreditList {
    fn default() -> Self {
        Self {
            ids: [0; CAPACITY],
            len: 0,
        }
    }
}
impl CreditList {
    pub(crate) fn len(&self) -> u16 {
        self.len
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.ids[..usize::from(self.len)]
    }
    pub(crate) fn first(&self) -> Option<u16> {
        self.as_slice().first().copied().map(u16::from)
    }
    pub(crate) fn contains(&self, id: u16) -> bool {
        u8::try_from(id).is_ok_and(|id| id != 0 && self.as_slice().contains(&id))
    }
    pub(crate) fn append(&mut self, id: u16) -> Result<(), String> {
        let id = u8::try_from(id)
            .ok()
            .filter(|id| *id != 0)
            .ok_or("Credit ID must be a physical identity from 1 to 255")?;
        if self.as_slice().contains(&id) {
            return Err("duplicate Credit identity".into());
        }
        if usize::from(self.len) == CAPACITY {
            return Err("Credit list capacity exceeded".into());
        }
        self.ids[usize::from(self.len)] = id;
        self.len += 1;
        Ok(())
    }
    pub(crate) fn remove(&mut self, id: u16) -> Result<(), String> {
        let index = self
            .as_slice()
            .iter()
            .position(|item| u16::from(*item) == id)
            .ok_or("Credit identity absent from BaseZone order")?;
        let end = usize::from(self.len);
        self.ids.copy_within(index + 1..end, index);
        self.len -= 1;
        self.ids[usize::from(self.len)] = 0;
        Ok(())
    }
    /// Capture the exact front-of-zone identities before constructing events.
    /// Payment rejects an unavailable prefix; transfer callers clamp first.
    pub(crate) fn prefix(&self, count: u16) -> Result<&[u8], String> {
        if count > self.len {
            return Err("Credit prefix exceeds available identities".into());
        }
        Ok(&self.as_slice()[..usize::from(count)])
    }
    /// Transfer handlers consume a clamped prefix, retaining donor and recipient
    /// order. Ownership changes are the caller's responsibility. Both lists stay
    /// unchanged on malformed identities or insufficient destination capacity.
    pub(crate) fn transfer_prefix_to(
        &mut self,
        recipient: &mut Self,
        amount: u16,
    ) -> Result<u16, String> {
        let count = amount.min(self.len);
        let mut donor = *self;
        let mut receiver = *recipient;
        for id in self.prefix(count)? {
            receiver.append(u16::from(*id))?;
            donor.remove(u16::from(*id))?;
        }
        *self = donor;
        *recipient = receiver;
        Ok(count)
    }
    /// Empty storage contributes nothing, preserving legacy hash neutrality.
    /// The caller supplies a player-specific tag and hashes exact-provenance
    /// state separately when the selected pool makes Credit semantics reachable.
    pub(crate) fn hash_values(&self, tag: u64, put: &mut impl FnMut(u64)) {
        if !self.is_empty() {
            put(tag);
            put(u64::from(self.len));
            for id in self.as_slice() {
                put(u64::from(*id));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaymentKind {
    PlayCard,
    Ability,
    CardEffect,
}
impl PaymentKind {
    pub(crate) fn prompt_title(self, title: &str) -> String {
        format!(
            "Use Credit tokens to pay for {title}{}",
            match self {
                Self::PlayCard => "",
                Self::Ability => "'s ability",
                Self::CardEffect => "'s effect",
            }
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaymentPlan {
    pub minimum: u16,
    pub maximum: u16,
    pub can_cancel: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuChoice {
    Use(u16),
    SelectAmount,
    Skip,
    Cancel,
}
impl MenuChoice {
    pub(crate) fn label(self) -> String {
        match self {
            Self::Use(n) => format!("Use {n} {}", if n == 1 { "Credit" } else { "Credits" }),
            Self::SelectAmount => "Select amount".into(),
            Self::Skip => "Pay costs without Credit tokens".into(),
            Self::Cancel => "Cancel".into(),
        }
    }
}
impl PaymentPlan {
    /// The pinned adjuster inspects only the FIRST token's blanking state.
    /// Do not substitute a per-token unblanked count, even for mixed input.
    pub(crate) fn available(credits: u16, first_is_blank: bool) -> u16 {
        if first_is_blank {
            0
        } else {
            credits
        }
    }
    /// `cost` is the remaining total at DefeatCredits_7, after preceding stages.
    /// None means no Credit prompt. An unaffordable whole charge is an error.
    pub(crate) fn new(
        cost: u16,
        ready: u16,
        credits: u16,
        first_is_blank: bool,
        can_cancel: bool,
    ) -> Result<Option<Self>, String> {
        let eligible = Self::available(credits, first_is_blank);
        if cost > ready.saturating_add(eligible) {
            return Err("resource payment is unaffordable".into());
        }
        if cost == 0 || eligible == 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            minimum: cost.saturating_sub(ready),
            maximum: eligible.min(cost),
            can_cancel,
        }))
    }
    pub(crate) fn choices(self) -> Vec<MenuChoice> {
        let mut out = vec![if self.minimum == self.maximum || self.maximum == 1 {
            MenuChoice::Use(self.maximum)
        } else {
            MenuChoice::SelectAmount
        }];
        if self.minimum == 0 {
            out.push(MenuChoice::Skip);
        }
        if self.can_cancel {
            out.push(MenuChoice::Cancel);
        }
        out
    }
    pub(crate) fn number_range(self) -> Option<std::ops::RangeInclusive<u16>> {
        self.choices()
            .contains(&MenuChoice::SelectAmount)
            .then(|| self.minimum.max(1)..=self.maximum)
    }
    pub(crate) fn validate_amount(self, amount: u16) -> Result<(), String> {
        if amount == 0 || amount < self.minimum || amount > self.maximum {
            return Err("Credit amount outside the offered payment range".into());
        }
        Ok(())
    }
}
pub(crate) const NUMBER_PROMPT_TITLE: &str = "Select amount of Credit tokens";

/// Only these complete, standalone ordinary registrations are initial producer
/// contracts. They remain data until the coordinator wires global payment rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Creation {
    pub amount: u16,
    pub event: bool,
}
pub(crate) fn creation_source(entry: &Value) -> Result<Option<Creation>, String> {
    fn marker(v: &Value) -> bool {
        v["kind"] == "createCreditToken"
            || match v {
                Value::Array(a) => a.iter().any(marker),
                Value::Object(o) => o.values().any(marker),
                _ => false,
            }
    }
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let reject = || {
        "Credit creation requires a complete standalone ordinary unit/event registration".to_owned()
    };
    let event = entry["baseClass"] == "EventCard";
    if entry["status"] != "supported"
        || entry["diagnostics"] != json!([])
        || !crate::static_ongoing::closed_source_context(entry)
        || !(event || entry["baseClass"] == "NonLeaderUnitCard")
        || entry["card"]["types"] != json!([if event { "event" } else { "unit" }])
        || !entry["card"]["unique"].is_boolean()
        || !entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        || (event
            && (!entry["card"]["hp"].is_null()
                || !entry["card"]["power"].is_null()
                || !entry["card"]["arena"].is_null()))
        || (!event
            && (!matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
                || !entry["card"]["hp"]
                    .as_u64()
                    .is_some_and(|n| (1..=1000).contains(&n))
                || !entry["card"]["power"].as_u64().is_some_and(|n| n <= 1000)))
        || entry["abilities"]["leader"] != json!([])
        || entry["abilities"]["unit"] != json!([])
        || !entry["abilities"].as_object().is_some_and(|sides| {
            sides
                .keys()
                .all(|key| ["leader", "unit", "card"].contains(&key.as_str()))
        })
        || !entry["piloting"].is_null()
        || !entry["attachCondition"].is_null()
        || ![
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|key| entry["card"][*key].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
    {
        return Err(reject());
    }
    let a = entry["abilities"]["card"]
        .as_array()
        .filter(|a| a.len() == 1)
        .ok_or_else(reject)?;
    let a = &a[0];
    let fields = a.as_object().ok_or_else(reject)?;
    if !fields
        .keys()
        .all(|key| ["trigger", "title", "costs", "effect"].contains(&key.as_str()))
        || a["trigger"] != if event { "event" } else { "whenPlayed" }
        || !a["title"].is_string()
        || a["costs"] != json!([])
    {
        return Err(reject());
    }
    let amount = a["effect"]["amount"]["value"]
        .as_u64()
        .filter(|n| if event { *n == 1 || *n == 3 } else { *n == 1 })
        .ok_or_else(reject)? as u16;
    if a["effect"]
        != json!({"kind":"createCreditToken","target":"ownPlayer","amount":{"kind":"const","value":amount},"destination":"base","creationTiming":"eventUpdate"})
    {
        return Err(reject());
    }
    let text = if !event {
        "When Played: Create a Credit token."
    } else if amount == 1 {
        "Create a Credit token."
    } else {
        "Create 3 Credit tokens."
    };
    if entry["card"]["text"] != text
        || entry["card"]["keywords"] != json!([])
        || entry["keywordValues"] != json!({})
    {
        return Err(reject());
    }
    Ok(Some(Creation { amount, event }))
}

/// Exact reviewed native implementation and metadata, including non-executable
/// source context. This recognizes provenance; it does not imply admission.
pub(crate) fn native_entry(entry: &Value) -> bool {
    let expected = json!({"title":"Credit","subtitle":"","cost":0,"hp":null,"power":null,"text":"While paying resources, you may defeat this token. If you do, pay 1 resource less.","deployBox":null,"epicAction":"","unique":false,"rules":"You may defeat Credit tokens while paying resources to play cards or use abilities.\n","upgradePower":null,"upgradeHp":null,"id":"8015500527","aspects":[],"traits":["supply"],"keywords":[],"types":["token"],"setId":{"set":"LAW"},"internalName":"credit"});
    crate::static_ongoing::closed_source_context(entry)
        && entry["setCode"] == "TOKEN_CREDIT"
        && entry["id"] == "8015500527"
        && entry["internalName"] == "credit"
        && entry["className"] == "CreditToken"
        && entry["baseClass"] == "TokenCard"
        && entry["sourcePath"] == "game/cards/07_LAW/tokens/CreditToken.ts"
        && entry["sourceHash"] == "08fbdaa83564a371d2988c05d5d9daa2f0c839605c1b8d806a1e8e2ac2a5c4db"
        && entry["card"] == expected
        && entry["keywordValues"] == json!({})
        && entry["abilities"] == json!({"leader":[],"unit":[],"card":[]})
        && entry["piloting"].is_null()
        && entry["attachCondition"].is_null()
}
