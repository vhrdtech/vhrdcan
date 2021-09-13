#![no_std]

pub mod id;
pub mod frame;
pub mod heap;

pub use id::FrameId;
pub use frame::{Frame, FrameRef};

#[cfg(feature = "serialization")]
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum Error {
    WrongLength
}

pub const EXTENDED_ID_ALL_BITS: u32 = 0x1FFFFFFF;
pub const STANDARD_ID_ALL_BITS: u16 = 0x7FF;
