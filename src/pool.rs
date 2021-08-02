// use crate::frame::Frame;
// use heapless::binary_heap::{BinaryHeap, Min};

// pub struct PriorityGroupingPoolItem<P: Ord, G: Eq, const FRAME_SIZE: usize> {
//     frame: Frame<FRAME_SIZE>,
//     priority: P,
//     group: G,
//     seq: u16,
// }
//
// pub struct PriorityGroupingPool<P: Ord, G: Eq, const FRAME_SIZE: usize, const POOL_SIZE: usize> {
//     heap: BinaryHeap<PriorityGroupingPoolItem<P, G, FRAME_SIZE>, Min, POOL_SIZE>,
//     seq: u16,
// }
//
// impl<P: Ord, G: Eq, const FRAME_SIZE: usize, const POOL_SIZE: usize> PriorityGroupingPool<P, G, FRAME_SIZE, POOL_SIZE> {
//     pub fn new() -> Self {
//         PriorityGroupingPool {
//             heap: BinaryHeap::new(),
//             seq: 0
//         }
//     }

    // pub fn push(&mut self, frame: Frame<FRAME_SIZE>, priority: P, group: G) {
    //     // if self.heap.capacity() - self.heap.len() > 0 {
    //         self.heap.push(PriorityGroupingPoolItem {
    //             frame,
    //             priority,
    //             group,
    //             seq: self.seq
    //         });
    //         self.seq = self.seq.wrapping_add(1);
    //     } else {
    //
    //     }
    // }
// }