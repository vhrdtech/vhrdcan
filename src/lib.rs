#![no_std]

pub mod id;
pub mod pool;
pub mod frame;

pub use id::FrameId;

pub use heapless;

#[cfg(feature = "serialization")]
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum Error {
    WrongLength
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
