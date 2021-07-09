#![no_std]

pub mod id;
pub use id::FrameId;
use core::cmp::{Ord, PartialOrd, Ordering};
use core::fmt;

use heapless::binary_heap::{BinaryHeap, Min};
use core::fmt::Formatter;

pub use heapless;

#[cfg(feature = "serialization")]
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum Error {
    WrongLength
}

pub enum FrameOrdering {
    OlderFirst,
    NewerFirst
}

pub struct RawFrameRef<'a> {
    pub id: FrameId,
    pub data: &'a [u8]
}
impl<'a> fmt::Debug for RawFrameRef<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
pub struct RawFrame {
    pub id: FrameId,
    pub data: [u8; 8],
    pub len: u8
}
impl RawFrame {
    pub fn new(id: FrameId, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None;
        }
        let mut data_copy = [0u8; 8];
        data_copy[0..data.len()].copy_from_slice(data);
        Some(RawFrame {
            id,
            data: data_copy,
            len: data.len() as u8
        })
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub fn as_raw_frame_ref(&self) -> RawFrameRef {
        RawFrameRef {
            id: self.id,
            data: self.data()
        }
    }
}
impl fmt::Debug for RawFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RawFrame{:?}", self.as_raw_frame_ref())
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Frame {
    id: FrameId,
    data: [u8; 8],
    len: u8,
    seq: i16
}
impl Frame {
    pub fn id(&self) -> FrameId {
        self.id
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
    pub fn sequence_number(&self) -> u16 {
        self.seq as u16
    }
    pub fn as_raw_frame_ref(&self) -> RawFrameRef {
        RawFrameRef {
            id: self.id,
            data: self.data()
        }
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Frame {
    fn cmp(&self, other: &Self) -> Ordering {
        let id_ord = self.id.cmp(&other.id);
        use Ordering::*;
        match id_ord {
            Greater => Greater,
            Less => Less,
            Equal => {
                let diff = self.seq.wrapping_sub(other.seq);
                diff.cmp(&0) // Frames with equal ID, but created earlier are processed first
            }
        }
    }
}
impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Frame[{}]{:?}", self.seq, self.as_raw_frame_ref())
    }
}

// pub trait FramePool {
//     type TargetFrame =
// }

pub struct FramePool {
    seq: i16
}
impl FramePool {
    pub fn new_pool() -> Self {
        FramePool {
            seq: 0
        }
    }
    pub fn new_frame(&mut self, id: FrameId, data: &[u8]) -> Result<Frame, Error> {
        if data.len() > 8 {
            return Err(Error::WrongLength);
        }
        let mut data_copy = [0u8; 8];
        data_copy[0..data.len()].copy_from_slice(data);
        self.seq = self.seq.wrapping_add(1);
        Ok(Frame{
            id,
            data: data_copy,
            len: data.len() as u8,
            seq: self.seq
        })
    }
    pub fn new_frame_from_raw(&mut self, raw_frame: RawFrame) -> Result<Frame, Error> {
        if raw_frame.len > 8 {
            return Err(Error::WrongLength);
        }
        self.seq = self.seq.wrapping_add(1);
        Ok(Frame {
            id: raw_frame.id,
            data: raw_frame.data,
            len: raw_frame.len,
            seq: self.seq
        })
    }
}

pub struct FrameHeap<N: heapless::ArrayLength<Frame>> {
    pub heap: BinaryHeap<Frame, N, Min>,
    pub pool: FramePool
}
impl<N: heapless::ArrayLength<Frame>> FrameHeap<N> {
    pub fn new() -> Self {
        FrameHeap {
            heap: BinaryHeap::new(),
            pool: FramePool::new_pool()
        }
    }
}

pub const EXTENDED_ID_ALL_BITS: u32 = 0x1FFFFFFF;
pub const STANDARD_ID_ALL_BITS: u16 = 0x7FF;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_heap() {
        let mut heap = FrameHeap::<heapless::consts::U32>::new();
        let r = heap.heap.push(heap.pool.new_frame(FrameId::standard(123).unwrap(), &[0, 1]).unwrap());
        assert!(r.is_ok());
    }
}
