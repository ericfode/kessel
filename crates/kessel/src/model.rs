use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MAX_CARDS: usize = 256;
pub const MAX_ZONE: usize = 64;
// Exact budget: 28,208 + native attack grants 64 + registration chronology 256
// + Disclose selection and alignment 16 + first-strike support flags (2 x 32 = 64)
// + phase restriction mask (Card 36→40, 256 cards = 1024)
// + until-end-of-attack stat grants (32 targets x 6 bytes + length = 194
// bytes, which reused the 2-byte alignment gap before the event window: net 192)
// + phase unit-defeat counters (2 controllers x 2 counters x u16 = 8, padded)
// + leader-deploy Plot window (PlotState: card/leader u16, pending u8,
//   programs u16, stage u8 + padding = 8, plus the resolved-instance bitmask
//   [u64; 4] = 32 that keeps one window from re-offering a resolved Plot)
// + Exploit pay-mode selection/continuation (Selection 12 + resume/settled 2,
//   padded to 16).
#[cfg(test)]
pub(crate) const EXPECTED_INLINE_STATE_BYTES: usize = 29_888;
pub type CardId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseHistory {
    pub exact: bool,
    /// Unit-defeat counters are only reconstructed from snapshots that record
    /// them; a legacy import keeps them unknown instead of inventing zeros.
    pub defeat_exact: bool,
    pub entered: [[u64; MAX_CARDS / 64]; 2],
    pub played: [[u64; MAX_CARDS / 64]; 2],
    pub play_counts: [u16; 2],
    /// `CardsDefeatedThisPhaseWatcher` unit entries by controller-at-defeat.
    /// Repeated defeats of one physical card are separate entries, so these are
    /// counters, not identity sets. Existential projections read `> 0`.
    pub defeated_counts: [u16; 2],
    /// Subset of `defeated_counts` whose entry had the pinned engine's
    /// while-attacking flag (the unit was the attacker of the current attack at
    /// defeat time and still in play).
    pub defeated_while_attacking_counts: [u16; 2],
}
impl Default for PhaseHistory {
    fn default() -> Self {
        Self {
            exact: true,
            defeat_exact: true,
            entered: [[0; MAX_CARDS / 64]; 2],
            played: [[0; MAX_CARDS / 64]; 2],
            play_counts: [0; 2],
            defeated_counts: [0; 2],
            defeated_while_attacking_counts: [0; 2],
        }
    }
}
impl PhaseHistory {
    pub fn contains(bits: &[[u64; MAX_CARDS / 64]; 2], seat: Seat, id: CardId) -> bool {
        bits[seat.index()][id as usize / 64] & (1u64 << (id as usize % 64)) != 0
    }
    pub fn entered(&mut self, seat: Seat, id: CardId) {
        // Legacy imports without these mechanics retain unknown history;
        // tracking a suffix would not make their phase history exact.
        if !self.exact {return;}
        self.entered[seat.index()][id as usize / 64] |= 1u64 << (id as usize % 64);
    }
    pub fn preflight_defeat(&self, seat: Seat) -> Result<(), String> {
        if !self.exact || !self.defeat_exact {return Ok(());}
        self.defeated_counts[seat.index()]
            .checked_add(1)
            .map(|_| ())
            .ok_or("phase unit-defeat count capacity exceeded".into())
    }
    /// Record one pinned-engine `DefeatedCardEntry` for an in-play unit. The
    /// caller has already established that the card is an in-arena unit, that
    /// it is the controller's while in play, and whether it was attacking.
    pub fn defeated(&mut self, seat: Seat, while_attacking: bool) -> Result<(), String> {
        if !self.exact || !self.defeat_exact {return Ok(());}
        self.preflight_defeat(seat)?;
        // Both counters are preflighted before either moves, so a refused
        // capture leaves the recorded history unchanged.
        if while_attacking {
            self.defeated_while_attacking_counts[seat.index()]
                .checked_add(1)
                .ok_or("phase while-attacking defeat count capacity exceeded")?;
        }
        self.defeated_counts[seat.index()] += 1;
        if while_attacking {
            self.defeated_while_attacking_counts[seat.index()] += 1;
        }
        Ok(())
    }
    pub fn preflight_play(&self, seat: Seat) -> Result<(), String> {
        if !self.exact {return Ok(());}
        self.play_counts[seat.index()]
            .checked_add(1)
            .map(|_| ())
            .ok_or("phase card-play count capacity exceeded".into())
    }
    pub fn played(&mut self, seat: Seat, id: CardId) -> Result<(), String> {
        if !self.exact {return Ok(());}
        self.preflight_play(seat)?;
        self.play_counts[seat.index()] += 1;
        self.played[seat.index()][id as usize / 64] |= 1u64 << (id as usize % 64);
        Ok(())
    }
    pub fn reset(&mut self) {
        let exact = self.exact;
        let defeat_exact = self.defeat_exact;
        *self = Self::default();
        self.exact = exact;
        self.defeat_exact = defeat_exact;
    }
    pub fn hash_values(&self, put: &mut impl FnMut(u64)) {
        if self.exact {
            put(0x5048415345484953);
            for bits in self.entered.iter().chain(&self.played) {
                for b in bits {
                    put(*b);
                }
            }
            for n in self.play_counts {
                put(n as u64);
            }
            // Zero defeat counters stay off the hash stream, so legacy/default
            // positions retain their historical hash.
            if self
                .defeated_counts
                .iter()
                .chain(&self.defeated_while_attacking_counts)
                .any(|n| *n != 0)
            {
                put(0x5048444546454154);
                for n in self.defeated_counts {
                    put(n as u64);
                }
                for n in self.defeated_while_attacking_counts {
                    put(n as u64);
                }
            }
        }
    }
    pub fn snapshot(&self) -> Value {
        let entries = |bits: &[[u64; MAX_CARDS / 64]; 2]| {
            let mut out = vec![];
            for seat in [Seat::P1, Seat::P2] {
                for id in 1..MAX_CARDS as u16 {
                    if Self::contains(bits, seat, id) {
                        out.push(json!({"card":format!("c{}",id-1),"playedBy":seat.as_str()}));
                    }
                }
            }
            out
        };
        let mut out =
            json!({"version":1,"entered":entries(&self.entered),"played":entries(&self.played),"playCounts":self.play_counts});
        // Same pattern as forceStateVersion: a new capability carries its own
        // marker, so consumers never infer defeat counters from an old export.
        if self.defeat_exact {
            out["defeatsVersion"] = json!(1);
            out["defeatedCounts"] = json!(self.defeated_counts);
            out["defeatedWhileAttackingCounts"] = json!(self.defeated_while_attacking_counts);
        }
        out
    }
    pub fn parse(v: &Value, next: CardId) -> Result<Self, String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Wire {
            version: u8,
            entered: Vec<Entry>,
            played: Vec<Entry>,
            play_counts: [u16; 2],
            #[serde(default)]
            defeats_version: Option<u8>,
            #[serde(default)]
            defeated_counts: [u16; 2],
            #[serde(default)]
            defeated_while_attacking_counts: [u16; 2],
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Entry {
            card: String,
            played_by: Seat,
        }
        let wire: Wire =
            serde_json::from_value(v.clone()).map_err(|e| format!("invalid phaseHistory: {e}"))?;
        if wire.version != 1 {
            return Err("unsupported phaseHistory version".into());
        }
        let mut out = Self::default();
        out.play_counts = wire.play_counts;
        match wire.defeats_version {
            None if wire.defeated_counts == [0; 2]
                && wire.defeated_while_attacking_counts == [0; 2] =>
            {
                out.defeat_exact = false;
            }
            None => return Err("defeat counts require an explicit defeatsVersion".into()),
            Some(1) => {
                out.defeat_exact = true;
                out.defeated_counts = wire.defeated_counts;
                out.defeated_while_attacking_counts = wire.defeated_while_attacking_counts;
            }
            Some(_) => return Err("unsupported defeatsVersion".into()),
        }
        for seat in [Seat::P1, Seat::P2] {
            if out.defeated_while_attacking_counts[seat.index()]
                > out.defeated_counts[seat.index()]
            {
                return Err("phaseHistory while-attacking defeats exceed unit defeats".into());
            }
        }
        for (entries, bits) in [
            (wire.entered, &mut out.entered),
            (wire.played, &mut out.played),
        ] {
            if entries.len() > 2 * (MAX_CARDS - 1) {
                return Err("phaseHistory entry capacity exceeded".into());
            }
            for entry in entries {
                let id = entry
                    .card
                    .strip_prefix('c')
                    .and_then(|v| v.parse::<CardId>().ok().and_then(|n| n.checked_add(1)))
                    .filter(|id| *id > 0 && *id < next && entry.card == format!("c{}", id - 1))
                    .ok_or("phaseHistory needs allocated canonical physical card IDs")?;
                if Self::contains(bits, entry.played_by, id) {
                    return Err("duplicate phaseHistory physical/controller identity".into());
                }
                bits[entry.played_by.index()][id as usize / 64] |= 1u64 << (id as usize % 64);
            }
        }
        for seat in [Seat::P1, Seat::P2] {
            let unique = out.played[seat.index()]
                .iter()
                .map(|n| n.count_ones())
                .sum::<u32>();
            if unique > out.play_counts[seat.index()] as u32
                || (unique == 0 && out.play_counts[seat.index()] != 0)
            {
                return Err("phaseHistory play count below distinct played identities".into());
            }
        }
        Ok(out)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Seat {
    P1,
    P2,
}
impl Seat {
    pub fn other(self) -> Self {
        if self == Self::P1 {
            Self::P2
        } else {
            Self::P1
        }
    }
    pub fn index(self) -> usize {
        if self == Self::P1 {
            0
        } else {
            1
        }
    }
    pub fn as_str(self) -> &'static str {
        if self == Self::P1 {
            "p1"
        } else {
            "p2"
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "p1" => Ok(Self::P1),
            "p2" => Ok(Self::P2),
            _ => Err(format!("invalid seat {s}")),
        }
    }
}
impl Default for Seat {
    fn default() -> Self {
        Self::P1
    }
}

/// Captured modified attacks have bounded storage, including grants retained by
/// the pinned engine when the optional attack is passed before declaration.
pub const MAX_SUPPORT_GRANTS: usize = 32;
pub const MAX_SUPPORT_PROGRAMS: usize = 80;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportGrant {
    pub target: CardId,
    pub donor: CardId,
    pub native_attack: bool,
    /// Until-end-of-attack "deals combat damage first" grant (first-strike).
    pub deals_combat_damage_first: bool,
    pub power: i16,
    pub programs: [u16; MAX_SUPPORT_PROGRAMS],
    pub len: u8,
    pub keywords: [u16; 9],
    pub presence: u16,
}
impl Default for SupportGrant {fn default()->Self {Self {target:0,donor:0,native_attack:false,deals_combat_damage_first:false,power:0,programs:[0;MAX_SUPPORT_PROGRAMS],len:0,keywords:[0;9],presence:0}}}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportState {
    pub grants: [SupportGrant; MAX_SUPPORT_GRANTS],
    pub len: u8,
    pub pending: bool,
    pub native_attack: bool,
    pub optional_attack: bool,
    pub context: ResolutionContext,
}
impl Default for SupportState {fn default()->Self {Self {grants:[SupportGrant::default();MAX_SUPPORT_GRANTS],len:0,pending:false,native_attack:false,optional_attack:false,context:ResolutionContext::default()}}}

/// One until-end-of-attack stat modification. The pinned engine creates one
/// unit-only `OngoingCardEffect` per target with `Duration.UntilEndOfAttack`
/// (`CardAttackLastingEffectSystem`); the values are the deltas resolved when
/// the ability applied, not composed live stats.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttackStatGrant {
    pub target: CardId,
    pub power: i16,
    pub hp: i16,
}
pub const MAX_ATTACK_STAT_GRANTS: usize = 32;
/// Until-end-of-attack stat effects share one lifecycle: every `AttackFlow`
/// cleanup removes all of them, and a target that leaves play loses its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackStats {
    pub grants: [AttackStatGrant; MAX_ATTACK_STAT_GRANTS],
    pub len: u8,
}
impl Default for AttackStats {
    fn default() -> Self {
        Self {
            grants: [AttackStatGrant::default(); MAX_ATTACK_STAT_GRANTS],
            len: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    pub ids: [CardId; MAX_ZONE],
    pub len: u8,
}
impl Default for Zone {
    fn default() -> Self {
        Self {
            ids: [0; MAX_ZONE],
            len: 0,
        }
    }
}
impl Zone {
    pub fn as_slice(&self) -> &[CardId] {
        &self.ids[..self.len as usize]
    }
    pub fn contains(&self, id: CardId) -> bool {
        self.as_slice().contains(&id)
    }
    pub fn push(&mut self, id: CardId) -> Result<(), String> {
        if self.len as usize == MAX_ZONE {
            return Err("zone capacity exceeded (64 cards)".into());
        }
        self.ids[self.len as usize] = id;
        self.len += 1;
        Ok(())
    }
    pub fn remove(&mut self, id: CardId) -> bool {
        if let Some(i) = self.as_slice().iter().position(|&c| c == id) {
            self.ids.copy_within(i + 1..self.len as usize, i);
            self.len -= 1;
            self.ids[self.len as usize] = 0;
            true
        } else {
            false
        }
    }
    pub fn pop_front(&mut self) -> Option<CardId> {
        let id = *self.as_slice().first()?;
        self.remove(id);
        Some(id)
    }
}

/// Global unit entry order in one arena, independent of controller. Unit-only
/// capacity follows the existing whole-card bound, not a per-player zone bound.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ArenaOrder {pub ids:[CardId;MAX_CARDS],pub len:u16}
impl Default for ArenaOrder {fn default()->Self {Self {ids:[0;MAX_CARDS],len:0}}}
impl ArenaOrder {
    pub fn as_slice(&self)->&[CardId] {&self.ids[..self.len as usize]}
    pub fn push(&mut self,id:CardId)->Result<(),String> {
        if id==0 || self.as_slice().contains(&id) {return Err("duplicate/invalid arena unit order entry".into());}
        if self.len as usize>=MAX_CARDS {return Err("global arena unit order capacity exceeded".into());}
        self.ids[self.len as usize]=id;self.len+=1;Ok(())
    }
    pub fn remove(&mut self,id:CardId) {
        if let Some(i)=self.as_slice().iter().position(|v|*v==id) {self.ids.copy_within(i+1..self.len as usize,i);self.len-=1;self.ids[self.len as usize]=0;}
    }
}

/// Physical entry/attachment chronology for currently registered printed unit
/// abilities and bounded ordinary-upgrade grants. Card IDs are 1..255.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationOrder {pub ids:[u8;MAX_CARDS],pub len:u16,pub exact:bool}
impl Default for RegistrationOrder {fn default()->Self {Self {ids:[0;MAX_CARDS],len:0,exact:true}}}
impl RegistrationOrder {
 pub fn parse(raw:&Value,next:CardId)->Result<Self,String> {
  let object=raw.as_object().ok_or("registrationOrder must be an object")?;
  if object.len()!=2||raw["version"]!=1||!object.contains_key("cards"){return Err("registrationOrder requires version 1 and cards only".into());}
  let mut actual=Self::default();
  for value in raw["cards"].as_array().ok_or("registrationOrder.cards must be an array")? {
   let text=value.as_str().ok_or("registrationOrder needs stable card IDs")?;
   let index=text.strip_prefix('c').and_then(|id|id.parse::<u16>().ok()).ok_or("registrationOrder needs stable card IDs")?;
   if text!=format!("c{index}")||index as usize>=MAX_CARDS-1||index+1>=next{return Err("registrationOrder card ID outside physical identity range".into());}
   actual.push(index+1)?;
  }Ok(actual)
 }
 pub fn as_slice(&self)->&[u8] {&self.ids[..self.len as usize]}
 pub fn push(&mut self,id:CardId)->Result<(),String> {
  let id=u8::try_from(id).map_err(|_|"registration card ID exceeds bounded domain")?;
  if id==0||self.as_slice().contains(&id){return Err("duplicate/invalid registration order entry".into());}
  if self.len as usize==MAX_CARDS{return Err("registration order capacity exceeded".into());}
  self.ids[self.len as usize]=id;self.len+=1;Ok(())
 }
 pub fn remove(&mut self,id:CardId) {if let Some(i)=self.as_slice().iter().position(|v|*v as CardId==id){self.ids.copy_within(i+1..self.len as usize,i);self.len-=1;self.ids[self.len as usize]=0;}}
 pub fn position(&self,id:CardId)->Option<usize> {self.as_slice().iter().position(|v|*v as CardId==id)}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub def: u16,
    pub present: bool,
    pub owner: Seat,
    pub damage: u16,
    pub phase_power: i32,
    pub phase_hp: i32,
    pub phase_keyword_values: [u16; 2],
    pub phase_keyword_presence: u8,
    pub phase_lost_keywords: u8,
    /// Action-phase prevent mask. Zero stays off the historical hash stream.
    pub phase_restrictions: u8,
    /// Current phase play/deploy/create qualification, independent of live Hidden.
    pub hidden_state: u8,
    pub exhausted: bool,
    pub deployed: bool,
    pub epic_used: bool,
    pub shield: CardId,
    pub experience: CardId,
    /// Token linked-list pointer; only token cards use it.
    pub next_token: CardId,
    pub parent: CardId,
    pub upgrades: CardId,
    pub next_upgrade: CardId,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Player {
    pub base: CardId,
    pub leader: CardId,
    pub hand: Zone,
    pub deck: Zone,
    pub resources: Zone,
    pub discard: Zone,
    pub ground: Zone,
    pub space: Zone,
    pub out: bool,
    pub force_id: CardId,
    pub force_held: bool,
}

/// Every engine prompt is retained, including confirmation and cancellation prompts.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    #[default]
    Window,
    LeaderMenu,
    AttackTarget,
    UpgradeTarget,
    VaderBase,
    SatineHeal,
    NoEffect,
    Resource,
    Unique,
    Ambush,
    TriggerOrder,
    ShieldOrder,
    AbilityMenu,
    AbilityTarget,
    AbilityOptional,
    AbilityNoEffect,
    AbilityDistribute,
    AbilityGroupShield,
    AbilityDiscard,
    DeathPlayer,
    DeathOrder,
    DeathBatch,
    Terminal,
    DiscloseSelect,
    DisclosePause,
    DiscloseReveal,
    Support,
    /// Optional prompt for a Plot keyword ability that was chosen in the
    /// leader-deploy trigger window: Trigger plays, Pass returns to the window.
    PlotOptional,
    /// Pay-mode prompt for an intrinsic Exploit play cost adjustment.
    ExploitChoice,
    /// Friendly-unit multi-select for an accepted Exploit adjustment.
    ExploitTarget,
}

/// Ordered target context retained across suspended event generation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Selection { pub ids: [CardId; 5], pub len: u8 }
impl Selection {
    pub fn hash_ids(&self)->impl Iterator<Item=CardId>+'_ { self.ids[..if self.len>3 {5}else{3}].iter().copied() }
    pub fn one(id: CardId) -> Self { Self { ids: [id, 0, 0, 0, 0], len: u8::from(id != 0) } }
    pub fn as_slice(&self) -> &[CardId] { &self.ids[..self.len as usize] }
    pub fn toggle(&mut self, id: CardId) -> Result<(), String> {
        if let Some(i) = self.ids[..self.len as usize].iter().position(|v| *v == id) {
            for j in i..self.len as usize - 1 { self.ids[j] = self.ids[j+1]; }
            self.len -= 1; self.ids[self.len as usize] = 0;
        } else { if self.len as usize >= self.ids.len() { return Err("selection capacity exceeded".into()); } self.ids[self.len as usize] = id; self.len += 1; }
        Ok(())
    }
}

/// Pinned-engine Exploit play-cost adjustment. `selection` retains the friendly
/// units chosen at the play-cost prompt (capacity 5 covers the catalogue's
/// maximum printed Exploit 4). `resume` records which continuation the event
/// payment boundary interrupted, because the engine offers the adjuster after
/// an event's early targets and before its cost is committed. `settled` marks
/// that the pay-mode choice was already answered for the pending payment, so a
/// resumed program instruction cannot offer the prompt twice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExploitState { pub selection: Selection, pub resume: u8, pub settled: bool }
/// Resume the interrupted event program (`run_program`).
pub const EXPLOIT_RESUME_PROGRAM: u8 = 0;
/// Commit the suspended event cost, then end a program with no resolvable effect.
pub const EXPLOIT_RESUME_END: u8 = 1;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PreparedEvent {
    pub op: u16, pub target: CardId, pub amount: u16,
    pub power: i32, pub hp: i32, pub token: CardId,
}
/// Uniform arena damage retains one ordered recipient and Shield context per event.
/// Both IDs are checked against the model's 256-card domain before compact storage.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UniformDamageRecord { pub target: u8, pub shield: u8 }
impl UniformDamageRecord {
    pub fn new(target:CardId,shield:CardId)->Result<Self,String> {
        if target==0 {return Err("uniform damage requires a real recipient".into());}
        Ok(Self {target:u8::try_from(target).map_err(|_|"uniform damage recipient exceeds card domain")?,
            shield:u8::try_from(shield).map_err(|_|"uniform damage Shield exceeds card domain")?})
    }
    pub fn target(self)->CardId {self.target as CardId}
    pub fn shield(self)->CardId {self.shield as CardId}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UniformDamageState {
    pub records:[UniformDamageRecord;MAX_CARDS], pub prevented:[u64;4],
    pub count:u16, pub active:bool,
}
impl Default for UniformDamageState {
    fn default()->Self {Self {records:[UniformDamageRecord::default();MAX_CARDS],prevented:[0;4],count:0,active:false}}
}
impl UniformDamageState {
    pub fn is_prevented(&self,index:usize)->bool {self.prevented[index/64]&(1u64<<(index%64))!=0}
    pub fn prevent(&mut self,index:usize) {self.prevented[index/64]|=1u64<<(index%64);}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventGroupState {
    pub active: bool, pub resolving: bool, pub replacement_done: bool, pub pc: u16, pub count: u8,
    pub discard_count:u8, pub discard_owner:Seat,
    pub shield_mask:u8, pub shield_count:u8, pub shield_priority:u8, pub initiator:Seat,
    pub shield_stash:[CardId;MAX_CARDS], pub shield_stash_len:u8,
    pub context: Selection, pub picks: Selection,
    pub stack: [Selection; 8], pub depth: u8,
    pub records: [PreparedEvent; 32],
    pub uniform_damage: UniformDamageState,
}
impl Default for EventGroupState {
    fn default() -> Self { Self { active:false, resolving:false, replacement_done:false, pc:0, count:0, discard_count:0, discard_owner:Seat::P1, shield_mask:0, shield_count:0, shield_priority:0, initiator:Seat::P1, shield_stash:[0;MAX_CARDS], shield_stash_len:0,
        context:Selection::default(), picks:Selection::default(), stack:[Selection::default();8], depth:0,
        records:[PreparedEvent::default();32], uniform_damage:UniformDamageState::default() } }
}
/// Result of one resolved instruction window. Identity is local to the active
/// resolution; nested death windows save and restore this payload with their parent.
/// Defeat counts include direct, successful unit defeat events only.
/// A zero payload clears the previous instruction's result; it is not a second
/// cumulative counter. Consumers must retain the source/program/pc scope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StepEventResult {
    pub source: CardId, pub program: u16, pub pc: u8,
    pub damage_dealt: u32, pub defeated_units: [u16; 2],
}
impl StepEventResult {
    pub fn for_state(s:&State)->Self {Self {source:s.source,program:s.ability_program,pc:s.ability_pc,..Self::default()}}
    pub fn belongs_to(self,source:CardId,program:u16,pc:u8)->bool {
        self.source==source && self.program==program && self.pc==pc
    }
}

/// Named views of the existing numeric state. Keep raw fields in State and
/// suspended contexts: their numeric hashing and legacy compatibility are stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TriggerContext { Action=0, Enter=1, Attack=2, ShieldOrder=3, DeathKeyword=4,
    /// The deploying player's leader entered the arena. Plot keyword abilities of
    /// cards in that player's resource zone watch this event (forceteki
    /// PlotAbility.buildPlotAbilityProperties, `onLeaderDeployed`).
    LeaderDeploy=5 }
impl TriggerContext {
    pub const fn raw(self)->u8 {self as u8}
    pub const fn from_raw(raw:u8)->Option<Self> {match raw {
        0=>Some(Self::Action),1=>Some(Self::Enter),2=>Some(Self::Attack),
        3=>Some(Self::ShieldOrder),4=>Some(Self::DeathKeyword),5=>Some(Self::LeaderDeploy),_=>None,
    }}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProgramContinuation { Finish=0, Triggers=1, Deaths=2,
    /// A Plot play of an event: after the event ability resolves, execute the
    /// Plot `ifYouDo` replacement and the played card's deferred triggers.
    Plot=3 }
impl ProgramContinuation {
    pub const fn raw(self)->u8 {self as u8}
    pub const fn from_raw(raw:u8)->Option<Self> {match raw {
        0=>Some(Self::Finish),1=>Some(Self::Triggers),2=>Some(Self::Deaths),3=>Some(Self::Plot),_=>None,
    }}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeathContinuation {
    Program=1, Triggers=2, Finish=3, PhaseExpiry=4, CombatAfterDamage=5,
    ParentDeaths=6, NativeAttackAfterEarlyEffects=7,
}
impl DeathContinuation {
    pub const fn raw(self)->u8 {self as u8}
    pub const fn from_raw(raw:u8)->Option<Self> {match raw {
        1=>Some(Self::Program),2=>Some(Self::Triggers),3=>Some(Self::Finish),
        4=>Some(Self::PhaseExpiry),5=>Some(Self::CombatAfterDamage),
        6=>Some(Self::ParentDeaths),7=>Some(Self::NativeAttackAfterEarlyEffects),_=>None,
    }}
}
pub const MAX_DEATH_TRIGGERS: usize = 64;
pub const MAX_DEATH_DEPTH: usize = 8;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeathTrigger { pub last_known_power:u16, pub grantor:CardId, pub source: CardId, pub program: u16, pub player: Seat, pub after_pc: u8, pub defer_depth: u8, pub keyword: u8 }
impl DeathTrigger {
    /// Registration chronology belongs to the physical grantor, while execution
    /// retains the host as source. Ordinary registrations use their own source.
    pub fn physical_source(self)->CardId {if self.grantor!=0 {self.grantor}else{self.source}}
}
/// A shared bounded trigger window, including attack, entry and defeat captures.
/// The historical name does not imply every entry is a card's WhenDefeated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeathWindow {
    pub entries: [DeathTrigger; MAX_DEATH_TRIGGERS], pub len: u8,
    pub initiator: Seat, pub player: Seat, pub player_chosen: bool,
    pub group_program: u16, pub group_source: CardId, pub group_keyword:u8, pub resolve_all: bool,
}
impl Default for DeathWindow {
    fn default() -> Self { Self { entries:[DeathTrigger::default();MAX_DEATH_TRIGGERS], len:0,
        initiator:Seat::P1, player:Seat::P1, player_chosen:false,
        group_program:0, group_source:0, group_keyword:0, resolve_all:false } }
}
/// One bounded property-factory evaluation, retained across its nested target prompt.
/// ResolutionContext saves/restores it around nested windows; ending a program or
/// clearing its action discards it. `sole` is the first maximum even for a tie;
/// `count`, rather than that field's name, determines uniqueness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MaximumCapture {pub power:i32,pub sole:CardId,pub count:u16}
impl MaximumCapture {
    /// Record candidates in their existing order. Zero-power candidates count;
    /// negative powers remain below the historical zero floor.
    pub fn observe(&mut self,power:i32,id:CardId)->Result<(),String> {
        if power>self.power {*self=Self {power,sole:id,count:1};}
        else if power==self.power {
            self.count=self.count.checked_add(1).ok_or("maximum tie count exceeds capacity")?;
            if self.sole==0 {self.sole=id;}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResolutionContext {
    pub active: Seat,
    pub decision: Decision,
    pub source: CardId,
    pub target: CardId,
    pub ambush: bool,
    pub after_attack_triggers: u8,
    pub attack_restore: u16,
    pub attack_source: CardId,
    pub attack_defender: CardId,
    /// Attack completion captured once; defeat tracks the original attacker even after re-entry.
    pub attack_end_captured: bool,
    pub attack_attacker_defeated: bool,
    pub attack_attacker_departed: bool,
    pub attack_defender_defeated: bool,
    pub attack_defender_departed: bool,
    pub attack_shield_opponent_first: bool,
    pub pending_triggers: u8,
    pub trigger_context: u8,
    pub pending_programs: u16,
    pub after_attack_programs: u16,
    pub ability_program: u16,
    pub ability_pc: u8,
    pub ability_target: CardId,
    pub maximum_capture: MaximumCapture,
    pub ability_selection: Selection,
    pub event_result: StepEventResult,
    pub last_known_source: CardId,
    pub last_known_power: u16,
    pub ability_result: u16,
    pub ability_succeeded: bool,
    pub ability_return: u8,
    pub ability_committed: bool,
}
impl ResolutionContext {
    pub fn capture(s:&State)->Self { Self { active:s.active, decision:s.decision, source:s.source, target:s.target, ambush:s.ambush, after_attack_triggers:s.after_attack_triggers, attack_restore:s.attack_restore, attack_source:s.attack_source,attack_defender:s.attack_defender,attack_end_captured:s.attack_end_captured,attack_attacker_defeated:s.attack_attacker_defeated,attack_attacker_departed:s.attack_attacker_departed,attack_defender_defeated:s.attack_defender_defeated,attack_defender_departed:s.attack_defender_departed, attack_shield_opponent_first:s.attack_shield_opponent_first, pending_triggers:s.pending_triggers, trigger_context:s.trigger_context, pending_programs:s.pending_programs, after_attack_programs:s.after_attack_programs, ability_program:s.ability_program, ability_pc:s.ability_pc, ability_target:s.ability_target, maximum_capture:s.maximum_capture, ability_selection:s.ability_selection, event_result:s.event_result,last_known_source:s.last_known_source,last_known_power:s.last_known_power,ability_result:s.ability_result, ability_succeeded:s.ability_succeeded, ability_return:s.ability_return, ability_committed:s.ability_committed } }
    pub fn restore(self,s:&mut State) { s.active=self.active; s.decision=self.decision; s.source=self.source; s.target=self.target; s.ambush=self.ambush; s.after_attack_triggers=self.after_attack_triggers; s.attack_restore=self.attack_restore; s.attack_source=self.attack_source;s.attack_defender=self.attack_defender;s.attack_end_captured=self.attack_end_captured;s.attack_attacker_defeated=self.attack_attacker_defeated;s.attack_attacker_departed=self.attack_attacker_departed;s.attack_defender_defeated=self.attack_defender_defeated;s.attack_defender_departed=self.attack_defender_departed; s.attack_shield_opponent_first=self.attack_shield_opponent_first; s.pending_triggers=self.pending_triggers; s.trigger_context=self.trigger_context; s.pending_programs=self.pending_programs; s.after_attack_programs=self.after_attack_programs; s.ability_program=self.ability_program; s.ability_pc=self.ability_pc; s.ability_target=self.ability_target;s.maximum_capture=self.maximum_capture; s.ability_selection=self.ability_selection; s.event_result=self.event_result;s.last_known_source=self.last_known_source;s.last_known_power=self.last_known_power;s.ability_result=self.ability_result; s.ability_succeeded=self.ability_succeeded; s.ability_return=self.ability_return; s.ability_committed=self.ability_committed; }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeathFrame { pub context: ResolutionContext, pub window: DeathWindow, pub resume: u8 }
impl DeathFrame {
    pub fn continuation(self)->Option<DeathContinuation> {DeathContinuation::from_raw(self.resume)}
}

/// Hand reveal selection is separate from continuation target selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiscloseState {pub player:Seat,pub selected:Selection}

/// A suspended leader-deploy trigger window while a Plot keyword ability plays
/// the chosen resource-zone card. PlotAbility.ts resolves the play through
/// PlayCardSystem (nested=false), so the play's event window passes its triggers
/// to the Plot ability's window: the top-of-deck resourcing runs before the
/// played card's When Played triggers, and those triggers resolve before the
/// leader's remaining deploy triggers are offered again. `leader_pending` and
/// `leader_programs` preserve exactly the outer window's still-unresolved
/// triggers. Zero everything else means no plot window is open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlotState {
    /// Plot card being offered (or already played while stage is Inner).
    pub card: CardId,
    /// Deploying leader whose trigger window is suspended. Nonzero means the
    /// leader-deploy trigger window is still open, including while a plot card
    /// resolves.
    pub leader: CardId,
    pub leader_pending: u8,
    pub leader_programs: u16,
    pub stage: u8,
    /// Plot instances already resolved (played or declined) in this window; the
    /// engine removes each triggered ability from its window when resolved.
    pub done: [u64; 4],
}
impl PlotState {
    /// Optional Trigger/Pass prompt for the chosen plot card.
    pub const OFFERED: u8 = 1;
    /// A plot upgrade waits for its attach target before entering play.
    pub const ATTACH: u8 = 2;
    /// The played card's own enter triggers resolve before the outer window.
    pub const INNER: u8 = 3;
    /// A plot play that created a unique replacement needs the choice first;
    /// the engine's uniqueness check runs inside the play window, before the
    /// Plot `ifYouDo` replacement.
    pub const UNIQUE: u8 = 4;
    pub fn active(self)->bool {self.stage!=0 || self.leader!=0}
    pub fn window_open(self)->bool {self.leader!=0}
    pub fn is_done(self, id:CardId)->bool {self.done[id as usize/64] & (1u64<<(id as usize%64)) != 0}
    pub fn mark_done(&mut self, id:CardId) {self.done[id as usize/64] |= 1u64<<(id as usize%64);}
}

/// All mutable game data is inline. Copying a position neither allocates nor shares mutable data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub disclose:DiscloseState,
    pub plot:PlotState,
    pub support: SupportState,
    /// Until-end-of-attack stat effects; cleared by the same attack-end sweep
    /// that removes the Support/native-attack grants.
    pub attack_stats: AttackStats,
    pub death_pending: DeathWindow,
    pub death_window: DeathWindow,
    pub death_frames: [DeathFrame; MAX_DEATH_DEPTH],
    pub death_depth: u8,
    pub cards: [Card; MAX_CARDS],
    pub next_id: CardId,
    pub players: [Player; 2],
    pub arena_unit_order:[ArenaOrder;2],
    pub registration_order:RegistrationOrder,
    /// False only for immutable legacy imports without ordinary upgrades.
    pub arena_order_exact:bool,
    pub hidden_history_exact:bool,
    pub phase_history:PhaseHistory,
    pub force_state_exact:bool,
    pub round: u16,
    pub regroup: bool,
    pub initiative: Seat,
    pub initiative_claimed: bool,
    pub prev_passed: bool,
    pub active: Seat,
    /// Action-phase priority is distinct from the owner of a prompted trigger.
    pub action_player: Option<Seat>,
    pub decision: Decision,
    pub source: CardId,
    // Internal entry-registration boundary; zero in every returned position.
    pub disabled_ongoing_source: CardId,
    pub target: CardId,
    pub ambush: bool,
    pub after_attack_triggers: u8,
    pub attack_restore: u16,
    pub attack_source: CardId,
    pub attack_defender: CardId,
    /// Attack completion captured once; defeat tracks the original attacker even after re-entry.
    pub attack_end_captured: bool,
    pub attack_attacker_defeated: bool,
    pub attack_attacker_departed: bool,
    pub attack_defender_defeated: bool,
    pub attack_defender_departed: bool,
    pub attack_shield_opponent_first: bool,
    pub pending_triggers: u8,
    pub trigger_context: u8,
    pub pending_programs: u16,
    pub after_attack_programs: u16,
    /// Zero means no IR continuation; otherwise program index plus one.
    pub ability_program: u16,
    pub ability_pc: u8,
    pub ability_target: CardId,
    pub maximum_capture: MaximumCapture,
    pub ability_selection: Selection,
    /// Intrinsic Exploit's play-cost prompt/selection, separate from ability targets.
    pub exploit: ExploitState,
    pub event_group: EventGroupState,
    pub event_result: StepEventResult,
    pub last_known_source: CardId,
    pub last_known_power: u16,
    pub ability_result: u16,
    pub ability_succeeded: bool,
    pub ability_return: u8,
    pub ability_committed: bool,
    pub resource_done: [bool; 2],
    pub resource_selected: [CardId; 2],
    /// 0 ongoing, 1 p1, 2 p2, 3 draw.
    pub outcome: u8,
}
impl Default for State {
    fn default() -> Self {
        Self {
            disclose:DiscloseState::default(),
            plot: PlotState::default(),
            support: SupportState::default(),
            attack_stats: AttackStats::default(),
            death_pending:DeathWindow::default(), death_window:DeathWindow::default(),
            death_frames:[DeathFrame::default();MAX_DEATH_DEPTH], death_depth:0,
            cards: [Card::default(); MAX_CARDS],
            next_id: 1,
            players: [Player::default(); 2],
            arena_unit_order:[ArenaOrder::default();2],registration_order:RegistrationOrder::default(),arena_order_exact:true,hidden_history_exact:false,force_state_exact:false,phase_history:PhaseHistory::default(),
            round: 1,
            regroup: false,
            initiative: Seat::P1,
            initiative_claimed: false,
            prev_passed: false,
            active: Seat::P1,
            action_player:Some(Seat::P1),
            decision: Decision::Window,
            source: 0,
            disabled_ongoing_source: 0,
            target: 0,
            ambush: false,
            after_attack_triggers: 0,
            attack_restore: 0,
            attack_source: 0,attack_defender:0,attack_end_captured:false,attack_attacker_defeated:false,attack_attacker_departed:false,attack_defender_defeated:false,attack_defender_departed:false,
            attack_shield_opponent_first: false,
            pending_triggers: 0,
            trigger_context: 0,
            pending_programs: 0,
            after_attack_programs: 0,
            ability_program: 0,
            ability_pc: 0,
            ability_target: 0,
            maximum_capture:MaximumCapture::default(),
            ability_selection: Selection::default(),
            exploit: ExploitState::default(),
            event_group: EventGroupState::default(),
            event_result:StepEventResult::default(),
            last_known_source:0,
            last_known_power:0,
            ability_result: 0,
            ability_succeeded: false,
            ability_return: 0,
            ability_committed: false,
            resource_done: [false; 2],
            resource_selected: [0; 2],
            outcome: 0,
        }
    }
}
impl State {
    pub fn trigger_context_is(&self,context:TriggerContext)->bool {self.trigger_context==context.raw()}
    pub fn set_trigger_context(&mut self,context:TriggerContext) {self.trigger_context=context.raw();}
    pub fn actor(&self) -> Option<Seat> {
        (!self.is_terminal()).then_some(self.active)
    }
    pub fn is_action_window(&self) -> bool {
        self.decision == Decision::Window && !self.is_terminal()
    }
    pub fn is_terminal(&self) -> bool {
        self.outcome != 0
    }
    pub fn winner(&self) -> Option<Seat> {
        match self.outcome {
            1 => Some(Seat::P1),
            2 => Some(Seat::P2),
            _ => None,
        }
    }
    pub fn card(&self, id: CardId) -> &Card {
        &self.cards[id as usize]
    }
    pub fn card_mut(&mut self, id: CardId) -> &mut Card {
        &mut self.cards[id as usize]
    }
    pub fn player(&self, seat: Seat) -> &Player {
        &self.players[seat.index()]
    }
    pub fn player_mut(&mut self, seat: Seat) -> &mut Player {
        &mut self.players[seat.index()]
    }
    /// Zobrist hashing over semantic features. SplitMix derives stable independent keys;
    /// no padding bytes, pointer values or allocation order enter the hash.
    pub fn zobrist(&self) -> u64 {
        fn key(i: u64, v: u64) -> u64 {
            let mut z = i.wrapping_mul(0x9e3779b97f4a7c15) ^ v.wrapping_mul(0xd6e8feb86659fd93);
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        }
        let mut h = 0;
        let mut i = 0;
        let mut put = |v: u64| {
            i += 1;
            h ^= key(i, v);
        };
        fn result_values(r:StepEventResult,lki_source:CardId,lki_power:u16,put:&mut impl FnMut(u64)) {
            if r!=StepEventResult::default()||lki_source!=0 {put(0x45564e54524553);for v in [r.source as u64,r.program as u64,r.pc as u64,r.damage_dealt as u64,r.defeated_units[0] as u64,r.defeated_units[1] as u64,lki_source as u64,lki_power as u64]{put(v);}}
        }
        fn context_values(c:&ResolutionContext,put:&mut impl FnMut(u64)) {
            if c.attack_end_captured || c.attack_attacker_defeated || c.attack_attacker_departed || c.attack_defender_defeated || c.attack_defender_departed {put(0x41545441434b454e);put(c.attack_end_captured as u64);put(c.attack_attacker_defeated as u64);put(c.attack_attacker_departed as u64);put(c.attack_defender_defeated as u64);put(c.attack_defender_departed as u64);}
            result_values(c.event_result,c.last_known_source,c.last_known_power,put);
            for v in [c.active.index() as u64,c.decision as u64,c.source as u64,c.target as u64,c.ambush as u64,c.after_attack_triggers as u64,c.attack_restore as u64,c.attack_source as u64,c.attack_defender as u64,c.attack_shield_opponent_first as u64,c.pending_triggers as u64,c.trigger_context as u64,c.pending_programs as u64,c.after_attack_programs as u64,c.ability_program as u64,c.ability_pc as u64,c.ability_target as u64,c.ability_result as u64,c.ability_succeeded as u64,c.ability_return as u64,c.ability_committed as u64,c.ability_selection.len as u64] {put(v);}
            for id in c.ability_selection.hash_ids() {put(id as u64);}
            if c.maximum_capture!=MaximumCapture::default(){put(0x4d4158434150);put(c.maximum_capture.power as u64);put(c.maximum_capture.sole as u64);put(c.maximum_capture.count as u64);}
        }
        fn window_values(w:&DeathWindow,put:&mut impl FnMut(u64)) {
            for v in [w.len as u64,w.initiator.index() as u64,w.player.index() as u64,w.player_chosen as u64,w.group_program as u64,w.group_source as u64,w.group_keyword as u64,w.resolve_all as u64] {put(v);}
            for e in w.entries {for v in [e.source as u64,e.program as u64,e.player.index() as u64,e.after_pc as u64,e.defer_depth as u64,e.keyword as u64] {put(v);}}
            for (i,e) in w.entries.iter().enumerate() {if e.last_known_power!=0 {put(0x4c4b49504f574552);put(i as u64);put(e.last_known_power as u64);}}
            for (i,e) in w.entries.iter().enumerate() {if e.grantor!=0 {put(0x4741494e4544);put(i as u64);put(e.grantor as u64);}}
        }
        if self.attack_end_captured || self.attack_attacker_defeated || self.attack_attacker_departed || self.attack_defender_defeated || self.attack_defender_departed {put(0x41545441434b454e);put(self.attack_end_captured as u64);put(self.attack_attacker_defeated as u64);put(self.attack_attacker_departed as u64);put(self.attack_defender_defeated as u64);put(self.attack_defender_departed as u64);}
        result_values(self.event_result,self.last_known_source,self.last_known_power,&mut put);
        put(self.death_depth as u64);
        window_values(&self.death_pending,&mut put);window_values(&self.death_window,&mut put);
        for f in &self.death_frames {put(f.resume as u64);context_values(&f.context,&mut put);window_values(&f.window,&mut put);}
        put(self.next_id as u64);
        put(self.arena_order_exact as u64);
        for arena in self.arena_unit_order {put(arena.len as u64);for id in arena.ids {put(id as u64);}}
        if self.registration_order.exact {put(0x5245474953544552);put(self.registration_order.len as u64);for &id in self.registration_order.as_slice(){put(id as u64);}}
        put(self.round as u64);
        if self.hidden_history_exact {put(1); }
        self.phase_history.hash_values(&mut put);
        if self.force_state_exact {put(1);}
        put(self.regroup as u64);
        put(self.initiative.index() as u64);
        put(self.initiative_claimed as u64);
        put(self.prev_passed as u64);
        if self.support.len != 0 || self.support.pending {
            put(0x535550504f5254);put(self.support.len as u64);put(self.support.pending as u64);
            for g in &self.support.grants[..self.support.len as usize] {
                if g.native_attack {put(0x4e41544956454154);put(g.power as u64);}
                if g.deals_combat_damage_first {put(0x4445414c53464952);}
                put(g.target as u64);put(g.donor as u64);put(g.len as u64);put(g.presence as u64);
                for &p in &g.programs[..g.len as usize] {put(p as u64);}
                for n in g.keywords {put(n as u64);}
            }
            if self.support.pending {
                if self.support.native_attack {put(0x4e41544956455041);put(self.support.optional_attack as u64);}
                context_values(&self.support.context,&mut put);
            }
        }
        // Attack-duration stat grants are resolution-internal: empty outside an
        // attack, so every zero grant leaves the historical hash stream untouched.
        if self.attack_stats.len != 0 {
            put(0x41545441434b5354);put(self.attack_stats.len as u64);
            for g in &self.attack_stats.grants[..self.attack_stats.len as usize] {
                put(g.target as u64);put(g.power as u16 as u64);put(g.hp as u16 as u64);
            }
        }
        put(self.active.index() as u64);
        put(self.action_player.map_or(0,|p|p.index() as u64+1));
        put(self.decision as u64);
        put(self.source as u64);
        put(self.target as u64);
        put(self.ambush as u64);
        put(self.after_attack_triggers as u64);
        put(self.attack_restore as u64);
        put(self.attack_source as u64);put(self.attack_defender as u64);
        put(self.attack_shield_opponent_first as u64);
        put(self.pending_triggers as u64);
        put(self.disabled_ongoing_source as u64);
        put(self.trigger_context as u64);
        put(self.outcome as u64);
        put(self.pending_programs as u64);
        put(self.after_attack_programs as u64);
        put(self.ability_program as u64);
        put(self.ability_pc as u64);
        put(self.ability_target as u64);
        put(self.ability_selection.len as u64);
        for id in self.ability_selection.hash_ids() { put(id as u64); }
        if self.maximum_capture!=MaximumCapture::default(){put(0x4d4158434150);put(self.maximum_capture.power as u64);put(self.maximum_capture.sole as u64);put(self.maximum_capture.count as u64);}
        let g = &self.event_group;
        put(g.discard_count as u64);put(g.discard_owner.index() as u64);
        put(g.active as u64); put(g.resolving as u64); put(g.replacement_done as u64); put(g.shield_mask as u64); put(g.shield_count as u64); put(g.shield_priority as u64); put(g.shield_stash_len as u64); for id in g.shield_stash { put(id as u64); } put(g.initiator.index() as u64); put(g.pc as u64); put(g.count as u64); put(g.depth as u64);
        for selection in [g.context, g.picks].into_iter().chain(g.stack) {
            put(selection.len as u64); for id in selection.hash_ids() { put(id as u64); }
        }
        for event in g.records { put(event.op as u64); put(event.target as u64); put(event.amount as u64);
            put(event.power as u64); put(event.hp as u64); put(event.token as u64);  }

        // Inactive default storage preserves the historical hash stream.
        let uniform=&g.uniform_damage;
        if uniform.active {
            put(uniform.active as u64);put(uniform.count as u64);
            for mask in uniform.prevented {put(mask);}
            for record in uniform.records {put(record.target as u64);put(record.shield as u64);}
        } else {debug_assert_eq!(*uniform,UniformDamageState::default(),"inactive uniform storage must be cleared");}

        if self.disclose!=DiscloseState::default() {put(0x64697363);put(self.disclose.player.index() as u64);put(self.disclose.selected.len as u64);for id in self.disclose.selected.hash_ids() {put(id as u64);}}
        // A suspended leader-deploy Plot window is internal prompt state; inactive
        // plot storage stays off the historical hash stream.
        if self.plot.active() {put(0x504c4f54);put(self.plot.card as u64);put(self.plot.leader as u64);put(self.plot.leader_pending as u64);put(self.plot.leader_programs as u64);put(self.plot.stage as u64);for word in self.plot.done {put(word);}}
        put(self.ability_result as u64);
        put(self.ability_succeeded as u64);
        put(self.ability_return as u64);
        put(self.ability_committed as u64);
        for n in 0..2 {
            put(self.resource_done[n] as u64);
            put(self.resource_selected[n] as u64);
        }
        // Qualification is observable only for current arena members. Build
        // membership once rather than scanning every zone for every card.
        let mut hidden_arena_members=[false;MAX_CARDS];
        if self.hidden_history_exact {
            for player in &self.players {for arena in [&player.ground,&player.space] {
                for &id in arena.as_slice() {hidden_arena_members[id as usize]=true;}
            }}
        }
        for (idx, c) in self
            .cards
            .iter()
            .enumerate()
            .take(self.next_id as usize)
            .skip(1)
        {
            put(idx as u64);
            put(c.experience as u64);
            put(c.phase_power as u64);
            put(c.phase_hp as u64);
            for amount in c.phase_keyword_values { put(amount as u64); }
            if c.hidden_state&24!=0 {put(0x535550);put((c.hidden_state&24) as u64);}
            put(c.phase_lost_keywords as u64);
            put(c.phase_keyword_presence as u64);
            if c.phase_restrictions != 0 {
                put(0x43525354);
                put(c.phase_restrictions as u64);
            }
            if self.hidden_history_exact {
                let in_arena=hidden_arena_members[idx];
                // Identity history outside the arena is inert in the admitted no-reentry domain.
                put((c.hidden_state & if in_arena {u8::MAX}else{!1}) as u64);
            }
            put(c.next_token as u64);
            put(c.parent as u64);put(c.upgrades as u64);put(c.next_upgrade as u64);
            put(c.def as u64
                | ((c.present as u64) << 16)
                | ((c.owner.index() as u64) << 17)
                | ((c.damage as u64) << 18)
                | ((c.exhausted as u64) << 34)
                | ((c.deployed as u64) << 35)
                | ((c.epic_used as u64) << 36)
                | ((c.shield as u64) << 37));
        }
        for p in &self.players {
            put(p.base as u64);
            put(p.leader as u64);
            put(p.out as u64);
            if self.force_state_exact {put(p.force_id as u64);put(p.force_held as u64);}
            for zone in [
                &p.hand,
                &p.deck,
                &p.resources,
                &p.discard,
                &p.ground,
                &p.space,
            ] {
                put(zone.len as u64);
                for &id in zone.as_slice() {
                    put(id as u64);
                }
            }
        }
        // Exploit prompt state is hash-relevant only while a play-cost choice is
        // pending; default storage stays off the historical hash stream.
        if self.exploit != ExploitState::default() {
            put(0x4558504c4f4954);
            put(self.exploit.resume as u64);
            put(self.exploit.settled as u64);
            put(self.exploit.selection.len as u64);
            for &id in &self.exploit.selection.ids[..self.exploit.selection.len as usize] {
                put(id as u64);
            }
        }
        h
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub kind: String,
    pub card: Option<String>,
    pub arg: Option<Value>,
    pub method: Option<String>,
    pub label: String,
}
impl Action {
    pub fn button(label: impl Into<String>, arg: impl Into<Value>) -> Self {
        Self {
            kind: "button".into(),
            card: None,
            arg: Some(arg.into()),
            method: None,
            label: label.into(),
        }
    }
    pub fn card(id: CardId, label: impl Into<String>) -> Self {
        Self {
            kind: "card".into(),
            card: Some(format!("c{}", id - 1)),
            arg: None,
            method: None,
            label: label.into(),
        }
    }
    pub fn key(&self) -> String {
        json!([self.kind, self.card, self.arg, self.method, self.label]).to_string()
    }
    pub fn id(&self) -> Option<CardId> {
        self.card
            .as_ref()?
            .strip_prefix('c')?
            .parse::<CardId>()
            .ok()?
            .checked_add(1)
    }
}

impl Card {
    pub fn hidden_qualified(&self)->bool {self.hidden_state & 1 != 0}
    pub fn set_hidden_qualified(&mut self,value:bool) {self.hidden_state=(self.hidden_state & !1)|u8::from(value);}
    pub fn phase_keyword_mask(&self)->u16 {self.phase_keyword_presence as u16 | if self.hidden_state&2!=0 {1<<8}else{0} | if self.hidden_state&8!=0 {1<<9}else{0}}
    pub fn lost_keyword_mask(&self)->u16 {self.phase_lost_keywords as u16 | if self.hidden_state&4!=0 {1<<8}else{0} | if self.hidden_state&16!=0 {1<<9}else{0}}
    pub fn add_phase_keyword(&mut self,index:usize) {if index==9 {self.hidden_state|=8;}else if index==8 {self.hidden_state|=2;}else{self.phase_keyword_presence|=1<<index;}}
    pub fn add_lost_keyword(&mut self,index:usize) {if index==9 {self.hidden_state|=16;}else if index==8 {self.hidden_state|=4;}else{self.phase_lost_keywords|=1<<index;}}
    pub fn clear_hidden_phase_effects(&mut self) {self.hidden_state &= 1;}
    pub fn phase_keyword_amount(&self, index: usize) -> u16 {
        if let Some(slot) = crate::ability_ir::numeric_keyword_slot(index) { self.phase_keyword_values[slot] } else { u16::from(self.phase_keyword_mask() & (1 << index) != 0) }
    }
}
