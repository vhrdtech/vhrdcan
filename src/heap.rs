use crate::Frame;
use core::cmp::Ordering;

pub trait MarkerTraits: Eq + PartialEq + Copy + Clone {}
impl<M> MarkerTraits for M where M: Eq + PartialEq + Copy + Clone {

}

pub trait GroupTraits: Eq + PartialEq + Copy + Clone {}
impl<M> GroupTraits for M where M: Eq + PartialEq + Copy + Clone {

}

#[derive(Copy, Clone)]
pub struct NoGrouping {}
impl PartialEq<Self> for NoGrouping {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl Eq for NoGrouping {}

#[derive(Eq, PartialEq, Copy, Clone)]
enum HeapElement<M: MarkerTraits, G: GroupTraits, const MTU: usize> {
    Hole,
    Filled(Frame<MTU>, i16, M, G)
}

impl<M: MarkerTraits, G: GroupTraits, const MTU: usize> Ord for HeapElement<M, G, MTU> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self {
            HeapElement::Hole => {
                match other {
                    // Hole's priority are equal, no need to move them around
                    HeapElement::Hole => { Equal }
                    // Any filled element priority is higher (less in can bus terminology)
                    HeapElement::Filled(_, _, _, _) => { Greater }
                }
            }
            HeapElement::Filled(self_frame, self_seq, _, _) => {
                match other {
                    // Any filled element priority is higher (less in can bus terminology)
                    HeapElement::Hole => { Less }
                    HeapElement::Filled(other_frame, other_seq, _, _) => {
                        match self_frame.cmp(other_frame) {
                            Less => { Less }
                            Equal => { self_seq.wrapping_sub(*other_seq).cmp(&0) }
                            Greater => { Greater }
                        }
                    }
                }
            }
        }
    }
}

impl<M: MarkerTraits, G: GroupTraits, const MTU: usize> PartialOrd for HeapElement<M, G, MTU> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq)]
pub enum SortOn {
    Push,
    Pop,
}

pub struct Heap<M: MarkerTraits, G: GroupTraits, const MTU: usize, const N: usize> {
    data: [HeapElement<M, G, MTU>; N],
    len: usize,
    hint_idx: usize,
    sort_on: SortOn,
    seq: i16,
}

impl<M: MarkerTraits, G: GroupTraits, const MTU: usize, const N: usize> Heap<M, G, MTU, N> {
    pub fn new(sort_on: SortOn) -> Self {
        Heap {
            data: [HeapElement::Hole; N],
            len: 0,
            hint_idx: 0,
            sort_on,
            seq: 0
        }
    }

    pub fn push(&mut self, frame: Frame<MTU>, marker: M, group: G) -> Result<usize, Frame<MTU>> {
        let mut replaced = 0;
        if self.len == N {
            if self.sort_on == SortOn::Push {
                self.data.sort_unstable();
                self.hint_idx = 0;
            }
            match self.data[N - 1] {
                HeapElement::Filled(stored_frame, _, _, _) => {
                    if frame < stored_frame {
                        let old_group = match self.data[N - 1] {
                            HeapElement::Hole => { unreachable!() }
                            HeapElement::Filled(_, _, _, og) => og
                        };
                        self.data[N - 1] = HeapElement::Filled(frame, self.seq, marker, group);
                        replaced = 1;

                        // Remove all frames from the same group as well
                        for elem in self.data.iter_mut() {
                            match elem {
                                HeapElement::Hole => {}
                                HeapElement::Filled(_, _, _, group) => {
                                    if old_group == *group {
                                        *elem = HeapElement::Hole;
                                    }
                                }
                            }
                        }
                    } else {
                        return Err(frame);
                    }
                }
                HeapElement::Hole => unreachable!()
            }
        } else {
            for elem in self.data.iter_mut() {
                if *elem == HeapElement::Hole {
                    *elem = HeapElement::Filled(frame, self.seq, marker, group);
                    break;
                }
            }
            self.len += 1;
        }
        if self.sort_on == SortOn::Push {
            self.data.sort_unstable();
            self.hint_idx = 0;
        }
        self.seq = self.seq.wrapping_add(1);

        Ok(replaced)
    }

    pub fn pop(&mut self) -> Option<(Frame<MTU>, M)> {
        if self.len == 0 {
            return None;
        }
        if self.sort_on == SortOn::Pop {
            self.data.sort_unstable();
            self.hint_idx = 0;
        }
        if self.hint_idx == N {
            return None;
        }
        match self.data[self.hint_idx] {
            HeapElement::Filled(frame, _, marker, _) => {
                self.data[self.hint_idx] = HeapElement::Hole;
                self.hint_idx += 1;
                self.len -= 1;
                Some((frame, marker))
            },
            HeapElement::Hole => None
        }
    }

    pub fn clear(&mut self) {
        for elem in self.data.iter_mut() {
            *elem = HeapElement::Hole;
        };
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct PlainHeap<M: MarkerTraits, const MTU: usize, const N: usize> {
    heap: Heap<M, NoGrouping, MTU, N>,
}
impl<M: MarkerTraits, const MTU: usize, const N: usize> PlainHeap<M, MTU, N> {
    pub fn new(sort_on: SortOn) -> Self {
        PlainHeap {
            heap: Heap::new(sort_on)
        }
    }

    pub fn push(&mut self, frame: Frame<MTU>, marker: M) -> Result<usize, Frame<MTU>> {
        self.heap.push(frame, marker, NoGrouping{})
    }

    pub fn pop(&mut self) -> Option<(Frame<MTU>, M)> {
        self.heap.pop()
    }

    pub fn clear(&mut self) {
        self.heap.clear();
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameId;

    #[test]
    fn check_sort_by_seq() {
        let mut heap = PlainHeap::<(), 8, 32>::new(SortOn::Push);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);

        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().0.data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().0.data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);

        let mut heap = PlainHeap::<(), 8, 32>::new(SortOn::Pop);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);

        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().0.data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().0.data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn check_sort_by_id_and_seq() {
        let mut heap = PlainHeap::<(), 8, 32>::new(SortOn::Push);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x1).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.push(Frame::new(FrameId::new_standard(0x1).unwrap(), &[1, 1]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 4);

        assert_eq!(heap.pop().unwrap().0.data(), &[1, 1]);
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop().unwrap().0.data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().0.data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn check_yield() {
        let mut heap = PlainHeap::<(), 8, 4>::new(SortOn::Push);
        let lower_prio = Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap();
        let higher_prio = Frame::new(FrameId::new_extended(0x12).unwrap(), &[4, 5, 6]).unwrap();
        assert_eq!(heap.push(lower_prio, ()), Ok(0));
        assert_eq!(heap.push(lower_prio, ()), Ok(0));
        assert_eq!(heap.push(lower_prio, ()), Ok(0));
        assert_eq!(heap.push(lower_prio, ()), Ok(0));
        assert!(heap.push(lower_prio, ()).is_err());
        assert_eq!(heap.len(), 4);
        assert_eq!(heap.push(higher_prio, ()), Ok(1));
        assert_eq!(heap.len(), 4);

        assert_eq!(heap.pop().unwrap().0.data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().0.data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }
}