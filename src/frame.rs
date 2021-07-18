use crate::id::FrameId;
use core::fmt;
use core::cmp::Ordering;

#[derive(Eq, PartialEq)]
pub struct FrameRef<'a> {
    pub id: FrameId,
    pub data: &'a [u8]
}
impl<'a> fmt::Debug for FrameRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !f.sign_minus() {
            write!(f, "FrameRef").ok();
        }
        let len = self.data.len();
        write!(f, "({:-?}, {}, ", self.id, len).ok();
        for i in 0..len {
            write!(f, "{:02x}", self.data[i]).ok();
            if i != len - 1 {
                write!(f, " ").ok();
            }
        }
        write!(f, ")")
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Frame<const N: usize> {
    pub id: FrameId,
    pub data: [u8; N],
    pub len: u16
}
impl<const N: usize> Frame<N> {
    pub fn new(id: FrameId, data: &[u8]) -> Option<Self> {
        if data.len() > N {
            return None;
        }
        let mut data_copy = [0u8; N];
        data_copy[0..data.len()].copy_from_slice(data);
        Some(Frame {
            id,
            data: data_copy,
            len: data.len() as u16
        })
    }

    pub fn new_move(id: FrameId, data: [u8; N], used: u8) -> Option<Frame<N>> {
        if data.len() > 8 {
            return None;
        }
        Some(Frame {
            id,
            data,
            len: used as u16
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub fn as_frame_ref(&self) -> FrameRef {
        FrameRef {
            id: self.id,
            data: self.data()
        }
    }
}
impl<const N: usize> fmt::Debug for Frame<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Frame{:-?}", self.as_frame_ref())
    }
}
impl<const N: usize> PartialOrd for Frame<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<const N: usize> Ord for Frame<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}