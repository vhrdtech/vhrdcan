#![no_std]

pub mod id;
pub use id::FrameId;
use core::cmp::{Ord, PartialOrd, Ordering};

use heapless::binary_heap::{BinaryHeap, Min};

#[derive(Debug)]
pub enum Error {
    WrongLength
}

#[derive(Eq, PartialEq)]
pub struct Frame {
    id: FrameId,
    data: [u8; 8],
    len: u8,
    seq: i16
}
impl Frame {

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
        if data.len() >= 8 {
            return Err(Error::WrongLength);
        }
        let mut data_copy = [0u8; 8];
        data_copy.copy_from_slice(data);
        self.seq.wrapping_add(1);
        Ok(Frame{
            id,
            data: data_copy,
            len: data.len() as u8,
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

