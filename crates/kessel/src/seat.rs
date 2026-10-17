//! Two-player identity and its stable wire representation; no game rules or state.
use serde::{Deserialize, Serialize};

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
