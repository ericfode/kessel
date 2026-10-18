//! Naming has three owners: the closed source proof installs one internal
//! instruction; its compulsory dropdown captures a title after unit payment;
//! this additive ledger guards every supported Play path until source departure.
//! These two when-played sources have no cost or prompt between choice and
//! application. Event pretarget naming, phase locks and cost increases need
//! separate adapters and remain rejected.
use super::*;
use crate::ability_ir::Instruction;

impl Engine {
    fn title_registry(&self) -> Option<&crate::named_play::Registry> {
        self.catalogue.as_ref()?.playable_card_titles.as_ref()
    }
    /// Mirrors Game.filterToLegalTitles: preserve registry order, ignoring names
    /// absent from it. Configure before creating/importing a position; existing
    /// positions with another naming universe are rejected by their fingerprint.
    pub fn set_legal_card_titles(&mut self, titles: &[String]) -> Result<(), String> {
        let registry = self
            .title_registry()
            .ok_or("catalogue has no reviewed naming registry")?;
        self.legal_card_titles = Some(
            registry
                .titles
                .iter()
                .enumerate()
                .filter(|(_, title)| titles.contains(title))
                .map(|(id, _)| id as u16)
                .collect(),
        );
        Ok(())
    }
    /// Evidence for an explicitly constructed new position in this game configuration.
    pub fn new_named_play_state(&self) -> Result<NamedPlayState, String> {
        Ok(NamedPlayState {
            registry: self.named_registry_fingerprint()?,
            ..NamedPlayState::default()
        })
    }
    fn named_registry_fingerprint(&self) -> Result<[u8; 32], String> {
        Ok(self
            .title_registry()
            .ok_or("catalogue has no reviewed naming registry")?
            .fingerprint(self.legal_card_titles.as_deref()))
    }
    pub(super) fn needs_named_play(&self) -> bool {
        self.selected_definition_any(|d| {
            d.programs.card.iter().any(|id| {
                self.program(*id)
                    .instructions
                    .iter()
                    .any(|i| matches!(i, Instruction::NameOpponentPlay))
            })
        })
    }
    fn state_needs_named_play(&self, s: &State) -> bool {
        (1..s.next_id).any(|id| {
            s.card(id).present
                && self.def(s, id).programs.card.iter().any(|p| {
                    self.program(*p)
                        .instructions
                        .iter()
                        .any(|i| matches!(i, Instruction::NameOpponentPlay))
                })
        })
    }
    pub(super) fn check_named_play(&self, s: &State) -> Result<(), String> {
        if usize::from(s.named_play.len) > MAX_NAMED_PLAY_LOCKS {
            return Err("named Play ledger capacity exceeded".into());
        }
        if self.needs_named_play() || self.state_needs_named_play(s) || s.named_play.len != 0 {
            if !s.named_play.exact {
                return Err("selected or imported pool requires exact named Play state".into());
            }
            if s.named_play.registry != self.named_registry_fingerprint()? {
                return Err("named Play registry configuration mismatch".into());
            }
        }
        Ok(())
    }
    pub(super) fn named_play_allowed(&self, s: &State, id: CardId) -> bool {
        if s.named_play.len == 0 {
            return true;
        }
        let Some(registry) = self.title_registry() else {
            return false;
        };
        let title = &self.def(s, id).info.title;
        !s.named_play.locks[..usize::from(s.named_play.len)]
            .iter()
            .any(|lock| {
                lock.recipient == s.card(id).owner
                    && self.in_arena(s, lock.source)
                    && registry.titles.get(usize::from(lock.title)) == Some(title)
            })
    }
    pub(super) fn naming_options(&self) -> Vec<(u16, &str)> {
        self.title_registry()
            .into_iter()
            .flat_map(|r| r.titles.iter().enumerate())
            .filter(|(id, _)| {
                self.legal_card_titles
                    .as_ref()
                    .is_none_or(|ids| ids.contains(&(*id as u16)))
            })
            .map(|(id, title)| (id as u16, title.as_str()))
            .collect()
    }
    /// Capture and application are atomic. A previously defeated source still
    /// names a card, but its while-source effect expires without an active lock.
    pub(super) fn install_named_play(&self, s: &mut State, title: u16) -> Result<(), String> {
        self.check_named_play(s)?;
        if !self.naming_options().iter().any(|(id, _)| *id == title) {
            return Err("title is outside the legal naming registry".into());
        }
        if !self.in_arena(s, s.source) {
            return Ok(());
        }
        let index = usize::from(s.named_play.len);
        if index == MAX_NAMED_PLAY_LOCKS {
            return Err("named Play ledger capacity exceeded".into());
        }
        s.named_play.locks[index] = NamedPlayLock {
            source: s.source,
            title,
            recipient: s.active.other(),
        };
        s.named_play.len += 1;
        Ok(())
    }
    pub(super) fn load_named_play(&self, s: &mut State, v: &Value) -> Result<(), String> {
        let Some(raw) = v.get("namedPlayState") else {
            s.named_play.exact = false;
            return self.check_named_play(s);
        };
        if raw.as_object().is_none_or(|o| o.len() != 3) || raw["version"] != 1 {
            return Err("unsupported named Play state version/fields".into());
        }
        let fingerprint = self.named_registry_fingerprint()?;
        let hex = fingerprint
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if raw["registry"] != hex {
            return Err("named Play registry fingerprint mismatch".into());
        }
        let locks = raw["locks"]
            .as_array()
            .ok_or("named Play state needs locks")?;
        if locks.len() > MAX_NAMED_PLAY_LOCKS {
            return Err("named Play ledger capacity exceeded".into());
        }
        s.named_play = NamedPlayState {
            registry: fingerprint,
            ..NamedPlayState::default()
        };
        for lock in locks {
            if lock.as_object().is_none_or(|o| o.len() != 3) {
                return Err("invalid named Play lock fields".into());
            }
            let source = cid(lock["source"].as_str().ok_or("named Play source missing")?)?;
            if source >= s.next_id
                || !s.card(source).present
                || !self.in_arena(s, source)
                || !self.def(s, source).programs.card.iter().any(|p| {
                    self.program(*p)
                        .instructions
                        .iter()
                        .any(|i| matches!(i, Instruction::NameOpponentPlay))
                })
            {
                return Err("named Play lock requires its retained arena naming source".into());
            }
            let recipient = Seat::parse(
                lock["recipient"]
                    .as_str()
                    .ok_or("named Play recipient missing")?,
            )?;
            let title = lock["title"].as_str().ok_or("named Play title missing")?;
            let title = self
                .naming_options()
                .into_iter()
                .find(|(_, t)| *t == title)
                .ok_or("named Play title is outside the legal registry")?
                .0;
            let index = usize::from(s.named_play.len);
            s.named_play.locks[index] = NamedPlayLock {
                source,
                title,
                recipient,
            };
            s.named_play.len += 1;
        }
        self.check_named_play(s)
    }
    pub(super) fn snapshot_named_play(&self, s: &State, v: &mut Value) {
        if s.named_play.exact
            && (self.needs_named_play() || self.state_needs_named_play(s) || s.named_play.len != 0)
        {
            let fingerprint = s
                .named_play
                .registry
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            let registry = self.title_registry().expect("bound naming registry");
            v["namedPlayState"] = json!({"version":1,"registry":fingerprint,"locks":s.named_play.locks[..usize::from(s.named_play.len)].iter().map(|lock|
                json!({"source":format!("c{}",lock.source-1),"title":registry.titles[usize::from(lock.title)],"recipient":lock.recipient.as_str()})).collect::<Vec<_>>()});
        }
        if s.ability_program != 0
            && self
                .program(s.ability_program - 1)
                .instructions
                .iter()
                .any(|i| matches!(i, Instruction::NameOpponentPlay))
            || s.death_frames[..usize::from(s.death_depth)]
                .iter()
                .any(|f| {
                    f.context.ability_program != 0
                        && self
                            .program(f.context.ability_program - 1)
                            .instructions
                            .iter()
                            .any(|i| matches!(i, Instruction::NameOpponentPlay))
                })
        {
            v["namedPlayPending"] = json!(true);
        }
    }
}
