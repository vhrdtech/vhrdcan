use core::cmp::Ordering;

#[derive(Eq, PartialEq)]
pub struct StandardId(u16);
impl StandardId {
    pub fn new(standard_id: u16) -> Option<StandardId> {
        if standard_id & (0b0001_1111 << 11) != 0 {
            None
        } else {
            Some(StandardId(standard_id))
        }
    }

    pub unsafe fn new_unchecked(standard_id: u16) -> StandardId {
        StandardId(standard_id)
    }
}

#[derive(Eq, PartialEq)]
pub struct ExtendedId(u32);
impl ExtendedId {
    pub fn new(extended_id: u32) -> Option<ExtendedId> {
        if extended_id & (0b111 << 29) != 0 {
            None
        } else {
            Some(ExtendedId(extended_id))
        }
    }

    pub unsafe fn new_unchecked(extended_id: u32) -> ExtendedId {
        ExtendedId(extended_id)
    }
}

#[derive(Eq, PartialEq)]
pub enum FrameId {
    Standard(StandardId),
    Extended(ExtendedId)
}
impl FrameId {
    pub fn standard(standard_id: u16) -> Option<FrameId> {
        StandardId::new(standard_id).map(|id| FrameId::Standard(id))
    }

    pub fn extended(extended_id: u32) -> Option<FrameId> {
        ExtendedId::new(extended_id).map(|id| FrameId::Extended(id))
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
                    FrameId::Extended(eid_r) => {
                        let eid_r_28_18 = (eid_r.0 >> 18) as u16;
                        if sid_l.0 == eid_r_28_18 {
                            return Ordering::Less; // Standard frame wins because of IDE dominant
                        }
                        sid_l.0.cmp(&eid_r_28_18)
                    }
                }
            },
            FrameId::Extended(eid_l) => {
                match other {
                    FrameId::Standard(sid_r) => {
                        let eid_l_28_18 = (eid_l.0 >> 18) as u16;
                        if eid_l_28_18 == sid_r.0 {
                            return Ordering::Greater; // Standard frame wins because of IDE dominant
                        }
                        eid_l_28_18.cmp(&sid_r.0)
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

#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::prelude::*;

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
        let sid0 = FrameId::standard(0).unwrap();
        let sid7 = FrameId::standard(7).unwrap();
        assert_eq!(sid0 < sid7, true);
        let eid0 = FrameId::extended(0).unwrap();
        let eid7 = FrameId::extended(7).unwrap();
        assert_eq!(sid0 != eid0, true);
        assert_eq!(eid0 < eid7, true);
        assert_eq!(sid0 < eid0, true);
        assert_eq!(eid0 > sid0, true);
        assert_eq!(sid7 > eid0, true);
    }
}