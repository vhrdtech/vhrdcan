use core::cmp::Ordering;
use core::fmt;
#[cfg(feature = "serialization")]
use serde::{Serialize, Deserialize};
use core::fmt::{Debug, Formatter};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct StandardId(u16);
impl StandardId {
    pub const fn new(standard_id: u16) -> Option<StandardId> {
        if standard_id & (0b0001_1111 << 11) != 0 {
            None
        } else {
            Some(StandardId(standard_id))
        }
    }

    pub fn inner(&self) -> u16 {
        self.0
    }

    pub unsafe fn new_unchecked(standard_id: u16) -> StandardId {
        StandardId(standard_id)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct ExtendedId(u32);
impl ExtendedId {
    pub const fn new(extended_id: u32) -> Option<ExtendedId> {
        if extended_id & (0b111 << 29) != 0 {
            None
        } else {
            Some(ExtendedId(extended_id))
        }
    }

    pub fn inner(&self) -> u32 {
        self.0
    }

    pub unsafe fn new_unchecked(extended_id: u32) -> ExtendedId {
        ExtendedId(extended_id)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum FrameId {
    Standard(StandardId),
    Extended(ExtendedId)
}
impl FrameId {
    pub const fn new_standard(standard_id: u16) -> Option<FrameId> {
        match StandardId::new(standard_id) {
            Some(id) => {
                Some(FrameId::Standard(id))
            },
            None => None
        }
    }

    pub const fn new_extended(extended_id: u32) -> Option<FrameId> {
        match ExtendedId::new(extended_id) {
            Some(id) => {
                Some(FrameId::Extended(id))
            },
            None => None
        }
    }
}
impl Ord for FrameId {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            FrameId::Standard(sid_l) => {
                match other {
                    FrameId::Standard(sid_r) => {
                        sid_l.0.cmp(&sid_r.0)
                    },
                    FrameId::Extended(_) => {
                        return Ordering::Less; // Standard frame wins because of IDE dominant
                    }
                }
            },
            FrameId::Extended(eid_l) => {
                match other {
                    FrameId::Standard(_) => {
                        return Ordering::Greater; // Standard frame wins because of IDE dominant
                    },
                    FrameId::Extended(eid_r) => {
                        eid_l.0.cmp(&eid_r.0)
                    }
                }
            }
        }
    }
}
impl PartialOrd for FrameId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Debug for FrameId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if !f.sign_minus() {
            let _ = write!(f, "FrameId(");
        }
        match self {
            FrameId::Standard(sid) => {
                let _ = write!(f, "{:#05X}", sid.0);
            },
            FrameId::Extended(eid) => {
                let _ = write!(f, "{:#010X}", eid.0);
            }
        }
        if !f.sign_minus() {
            write!(f, ")")
        } else {
            write!(f, "")
        }
    }
}
impl hash32::Hash for FrameId {
    fn hash<H: hash32::Hasher>(&self, state: &mut H) {
        match *self {
            FrameId::Standard(sid) => {
                state.write(&sid.0.to_le_bytes())
            }
            FrameId::Extended(eid) => {
                state.write(&eid.0.to_le_bytes())
            }
        }
    }
}

// #[cfg(test)]
// extern crate std;
// #[cfg(test)]
// use std::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_ids() {
        let sid = StandardId::new(123);
        assert!(sid.is_some());
        let sid = StandardId::new(0b00001000_00000000);
        assert!(sid.is_none());
        let eid = ExtendedId::new(123);
        assert!(eid.is_some());
        let eid = ExtendedId::new(0x20000000);
        assert!(eid.is_none());
        let sid0 = FrameId::new_standard(0).unwrap();
        let sid1 = FrameId::new_standard(1).unwrap();
        let sid7 = FrameId::new_standard(7).unwrap();
        assert_eq!(sid0 < sid7, true);
        let eid0 = FrameId::new_extended(0).unwrap();
        let eid1 = FrameId::new_extended(1).unwrap();
        let eid7 = FrameId::new_extended(7).unwrap();
        assert_eq!(sid0 != eid0, true);
        assert_eq!(eid0 < eid7, true);
        assert_eq!(sid0 < eid0, true);
        assert_eq!(sid1 < eid1, true);
        assert_eq!(eid0 > sid0, true);
        assert_eq!(sid7 < eid0, true);
    }
}