//! Registration-object ownership prototype, independent of the engine position.
//! A handle stays attached to queued contexts after its source changes zones.
//! Only registered current instances receive RoundEnded resets. Unreferenced
//! retired slots can be collected; quiescent normalization removes allocation
//! history without changing any observable use budget.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Handle(u16);
impl Handle {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) fn raw(self) -> u16 {
        self.0
    }
}
const CURRENT: u8 = 1;
const RETIRED: u8 = 2;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Instance {
    pub source_index: u8,
    pub registration_ordinal: u8,
    pub used_by_player: u8,
    lifecycle: u8,
}
// One native limited registration per physical card in these complete five
// profiles. Root handles cover both live windows, eight saved windows, active
// resolution, eight saved resolution contexts and Support's saved context.
// This conservative bound deliberately does not assume unique root references.
pub(crate) const PHYSICAL_CARD_SLOTS: usize = crate::model::MAX_CARDS;
const _: [();256]=[();PHYSICAL_CARD_SLOTS];
pub(crate) const TRIGGERS_PER_WINDOW: usize = crate::model::MAX_DEATH_TRIGGERS;
pub(crate) const SAVED_DEATH_FRAMES: usize = crate::model::MAX_DEATH_DEPTH;
pub(crate) const MAX_NATIVE_INSTANCES: usize = (PHYSICAL_CARD_SLOTS - 1)
    + (2 + SAVED_DEATH_FRAMES) * TRIGGERS_PER_WINDOW
    + 2
    + SAVED_DEATH_FRAMES;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ledger {
    pub(crate) entries: [Instance; MAX_NATIVE_INSTANCES],
    pub(crate) len: u16,
}
impl Default for Ledger {
    fn default() -> Self {
        Self {
            entries: [Instance::default(); MAX_NATIVE_INSTANCES],
            len: 0,
        }
    }
}
impl Ledger {
    fn index(&self, handle: Handle) -> Result<usize, String> {
        let index = usize::from(
            handle
                .0
                .checked_sub(1)
                .ok_or("missing captured round-limit instance")?,
        );
        if index >= usize::from(self.len) || self.entries[index].lifecycle == 0 {
            return Err("unknown captured round-limit instance".into());
        }
        Ok(index)
    }
    pub(crate) fn instance(&self, handle: Handle) -> Result<Instance, String> {
        Ok(self.entries[self.index(handle)?])
    }
    pub(crate) fn current(&self,source:u16,ordinal:u8)->Option<Handle> {
        self.entries[..usize::from(self.len)].iter().position(|e|e.lifecycle==CURRENT&&u16::from(e.source_index)==source&&e.registration_ordinal==ordinal).map(|i|Handle((i+1) as u16))
    }
    pub(crate) fn native(&mut self, source: u16, ordinal: u8) -> Result<Handle, String> {
        if ordinal>1 {return Err("unsupported native round-limit registration ordinal".into());}
        let source = u8::try_from(source)
            .ok()
            .filter(|source| *source != 0)
            .ok_or("round-limit source exceeds physical card capacity")?;
        if let Some(index) = self.entries[..usize::from(self.len)]
            .iter()
            .position(|entry| {
                entry.lifecycle == CURRENT
                    && entry.source_index == source
                    && entry.registration_ordinal == ordinal
            })
        {
            return Ok(Handle((index + 1) as u16));
        }
        if self.entries[..usize::from(self.len)].iter().any(|entry|entry.lifecycle==CURRENT&&entry.source_index==source) {return Err("multiple native limited registrations exceed reviewed whole-card profile".into());}
        let index = self.entries[..usize::from(self.len)]
            .iter()
            .position(|entry| entry.lifecycle == 0)
            .unwrap_or(usize::from(self.len));
        if index == MAX_NATIVE_INSTANCES {
            return Err("round-limit registration-instance capacity exceeded".into());
        }
        if index == usize::from(self.len) {
            self.len += 1;
        }
        self.entries[index] = Instance {
            source_index: source,
            registration_ordinal: ordinal,
            used_by_player: 0,
            lifecycle: CURRENT,
        };
        Ok(Handle((index + 1) as u16))
    }
    pub(crate) fn available(&self, handle: Handle, player_index: u8) -> Result<bool, String> {
        if player_index > 1 {
            return Err("invalid round-limit player".into());
        }
        Ok(self.instance(handle)?.used_by_player & (1 << player_index) == 0)
    }
    /// Called once at the source-validated resolver handler boundary, not when
    /// the event is captured, an optional prompt opens, or an effect succeeds.
    pub(crate) fn consume(&mut self, handle: Handle, player_index: u8) -> Result<(), String> {
        if !self.available(handle, player_index)? {
            return Err("round limit already used by this player".into());
        }
        let index = self.index(handle)?;
        self.entries[index].used_by_player |= 1 << player_index;
        Ok(())
    }
    /// Mirrors resetLimitForNewZone: the old shared limit is unregistered and
    /// retained contexts keep it. A later native() allocates the fresh clone.
    pub(crate) fn retire_native(&mut self, source: u16) -> Result<(), String> {
        let source = u8::try_from(source)
            .ok()
            .filter(|source| *source != 0)
            .ok_or("round-limit source exceeds physical card capacity")?;
        for entry in &mut self.entries[..usize::from(self.len)] {
            if entry.lifecycle == CURRENT && entry.source_index == source {
                entry.lifecycle = RETIRED;
            }
        }
        Ok(())
    }
    pub(crate) fn round_ended(&mut self) {
        for entry in &mut self.entries[..usize::from(self.len)] {
            if entry.lifecycle == CURRENT {
                entry.used_by_player = 0;
            }
        }
    }
    pub(crate) fn collect(&mut self, roots: &[Handle]) -> Result<(), String> {
        // Validate every root before mutation; malformed captured handles never
        // silently become a fresh budget or free a shared retired instance.
        let mut marked=[false;MAX_NATIVE_INSTANCES];
        for handle in roots {marked[self.index(*handle)?]=true;}
        for index in 0..usize::from(self.len) {
            if self.entries[index].lifecycle == RETIRED
                && !marked[index]
            {
                self.entries[index] = Instance::default();
            }
        }
        while self.len > 0 && self.entries[usize::from(self.len) - 1].lifecycle == 0 {
            self.len -= 1;
        }
        Ok(())
    }
    /// Caller must prove there are no pending handles before invoking this.
    pub(crate) fn normalize_quiescent(&mut self, roots: &[Handle]) -> Result<(), String> {
        if !roots.is_empty() {
            return Err("round-limit normalization requires a quiescent position".into());
        }
        let mut live: Vec<_> = self.entries[..usize::from(self.len)]
            .iter()
            .copied()
            .filter(|entry| entry.lifecycle == CURRENT && entry.used_by_player != 0)
            .collect();
        live.sort_by_key(|entry| (entry.source_index, entry.registration_ordinal));
        self.entries.fill(Instance::default());
        self.entries[..live.len()].copy_from_slice(&live);
        self.len = live.len() as u16;
        Ok(())
    }
    /// Default-zero omission is deliberate. Runtime integration must hash the
    /// captured handle graph separately while contexts are pending.
    pub(crate) fn hash_values(&self, mut put: impl FnMut(u64)) {
        if self.len == 0 {
            return;
        }
        put(0x524f554e444c494d);
        for (index, entry) in self.entries[..usize::from(self.len)].iter().enumerate() {
            if entry.lifecycle != 0 {
                put((index + 1) as u64);
                put(u64::from(entry.source_index));
                put(u64::from(entry.registration_ordinal));
                put(u64::from(entry.used_by_player));
                put(u64::from(entry.lifecycle));
            }
        }
    }
}

/// Per-physical damage-event generation facts. A known false differs from a
/// missing capture; neither replacement nor a preceding handler can relatch it.
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]
pub struct DamageFacts {generated:[u64;4],will_defeat:[u64;4]}
impl DamageFacts {
    pub(crate) fn capture(&mut self,index:usize,will_defeat:bool)->Result<(),String> {
        if index>=256 {return Err("damage event fact capacity exceeded".into());}
        let bit=1u64<<(index%64);self.generated[index/64]|=bit;
        if will_defeat {self.will_defeat[index/64]|=bit;}else{self.will_defeat[index/64]&=!bit;}
        Ok(())
    }
    pub(crate) fn get(&self,index:usize)->Result<bool,String> {
        if index>=256||self.generated[index/64]&(1u64<<(index%64))==0 {return Err("captured damage event willDefeat is missing".into());}
        Ok(self.will_defeat[index/64]&(1u64<<(index%64))!=0)
    }
    pub(crate) fn hash_values(self,mut put:impl FnMut(u64)) {
        if self==Self::default(){return;}put(0x444d474556464143);for n in self.generated {put(n);}for n in self.will_defeat {put(n);}
    }
}
