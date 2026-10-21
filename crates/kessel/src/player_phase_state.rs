//! Phase-retained applications. Slot identities never move/reuse before expiry.
use crate::model::CardId;
use crate::player_phase_contract::Kind;
pub(crate) const CAPACITY: usize = 64;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SlotId(u8);
impl SlotId {
    pub(crate) fn checked(index: usize) -> Result<Self, String> {
        if index >= CAPACITY {
            return Err("unsupported player phase application capacity (64)".into());
        }
        Ok(Self((index + 1) as u8))
    }
    pub(crate) fn index(self) -> Result<usize, String> {
        if self.0 == 0 {
            return Err("missing player phase slot identity".into());
        }
        Ok(usize::from(self.0 - 1))
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Row {
    pub source: CardId,
    pub program: u16,
    kind: u8,
    flags: u8,
}
impl Row {
    pub(crate) fn new(
        source: CardId,
        program: u16,
        kind: Kind,
        target: u8,
        limit_player: u8,
        used: u8,
    ) -> Result<Self, String> {
        if source == 0 || target > 1 || limit_player > 1 || used > 3 {
            return Err("invalid player phase row".into());
        }
        Ok(Self {
            source,
            program,
            kind: if kind == Kind::ImperialDiscount { 1 } else { 2 },
            flags: target | (limit_player << 1) | (used << 2),
        })
    }
    pub(crate) fn kind(self) -> Kind {
        if self.kind == 1 {
            Kind::ImperialDiscount
        } else {
            Kind::LowPrintedPowerReady
        }
    }
    pub(crate) fn target(self) -> u8 {
        self.flags & 1
    }
    pub(crate) fn limit_player(self) -> u8 {
        (self.flags >> 1) & 1
    }
    pub(crate) fn used(self) -> u8 {
        (self.flags >> 2) & 3
    }
    pub(crate) fn available(self) -> bool {
        self.used() & (1 << self.limit_player()) == 0
    }
    fn consume(&mut self) {
        self.flags |= 1 << (2 + self.limit_player());
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Mask(pub [u32; 2]);
impl Mask {
    pub(crate) fn insert(&mut self, id: SlotId) -> Result<(), String> {
        let n = id.index()?;
        self.0[n / 32] |= 1 << (n % 32);
        Ok(())
    }
    pub(crate) fn contains(self, n: usize) -> bool {
        self.0[n / 32] & (1 << (n % 32)) != 0
    }
    pub(crate) fn count(self) -> u16 {
        (self.0[0].count_ones() + self.0[1].count_ones()) as u16
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ledger {
    pub rows: [Row; CAPACITY],
    pub len: u8,
    pub wire_present: bool,
}
impl Default for Ledger {
    fn default() -> Self {
        Self {
            rows: [Row::default(); CAPACITY],
            len: 0,
            wire_present: false,
        }
    }
}
impl Ledger {
    pub(crate) fn preflight(&self, additional: usize) -> Result<(), String> {
        if usize::from(self.len) + additional > CAPACITY {
            return Err("unsupported player phase application capacity (64)".into());
        }
        Ok(())
    }
    pub(crate) fn append(&mut self, row: Row) -> Result<SlotId, String> {
        self.preflight(1)?;
        let id = SlotId::checked(self.len as usize)?;
        self.rows[id.index()?] = row;
        self.len += 1;
        Ok(id)
    }
    pub(crate) fn capture(&self, kind: Kind, player: u8) -> Mask {
        let mut m = Mask::default();
        for n in 0..usize::from(self.len) {
            let r = self.rows[n];
            if r.kind() == kind && r.target() == player && r.available() {
                m.insert(SlotId::checked(n).expect("bounded slot"))
                    .expect("bounded slot");
            }
        }
        m
    }
    /// Consume every captured row, with no late source/liveness/availability test.
    pub(crate) fn consume(&mut self, mask: Mask) -> Result<(), String> {
        if mask.count()==0 {return Ok(());}
        for n in usize::from(self.len)..CAPACITY {
            if mask.contains(n) {
                return Err("player phase capture references absent row".into());
            }
        }
        for n in 0..usize::from(self.len) {
            if mask.contains(n) {
                self.rows[n].consume();
            }
        }
        Ok(())
    }
    pub(crate) fn expire(&mut self) {
        let presence = self.wire_present;
        *self = Self::default();
        self.wire_present = presence;
    }
    pub(crate) fn hash_values(&self, put: &mut impl FnMut(u64)) {
        if self.len == 0 {
            return;
        }
        put(0x5048415345504c);
        put(self.len as u64);
        for r in &self.rows[..usize::from(self.len)] {
            put(r.source as u64);
            put(r.program as u64);
            put(r.kind as u64);
            put(r.flags as u64);
        }
    }
}
