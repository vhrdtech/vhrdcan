use crate::Frame;
use core::cmp::Ordering;

#[derive(Eq, PartialEq, Copy, Clone)]
enum HeapElement<M: Eq + PartialEq + Copy + Clone, const MTU: usize> {
    Hole,
    Filled(Frame<MTU>, i16, M)
}

impl<M: Eq + PartialEq + Copy + Clone, const MTU: usize> Ord for HeapElement<M, MTU> {
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;
        match self {
            HeapElement::Hole => {
                match other {
                    // Hole's priority are equal, no need to move them around
                    HeapElement::Hole => { Equal }
                    // Any filled element priority is higher (less in can bus terminology)
                    HeapElement::Filled(_, _, _) => { Greater }
                }
            }
            HeapElement::Filled(self_frame, self_seq, _) => {
                match other {
                    // Any filled element priority is higher (less in can bus terminology)
                    HeapElement::Hole => { Less }
                    HeapElement::Filled(other_frame, other_seq, _) => {
                        match self_frame.cmp(other_frame) {
                            Less => { Less }
                            Equal => { self_seq.cmp(other_seq) }
                            Greater => { Greater }
                        }
                    }
                }
            }
        }
    }
}

impl<M: Eq + PartialEq + Copy + Clone, const MTU: usize> PartialOrd for HeapElement<M, MTU> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq)]
pub enum SortOn {
    Push,
    Pop,
}

pub struct Heap<M: Eq + PartialEq + Copy + Clone, const MTU: usize, const N: usize> {
    data: [HeapElement<M, MTU>; N],
    len: usize,
    hint_idx: usize,
    sort_on: SortOn,
    seq: i16,
}

impl<M: Eq + PartialEq + Copy + Clone, const MTU: usize, const N: usize> Heap<M, MTU, N> {
    pub fn new(sort_on: SortOn) -> Self {
        Heap {
            data: [HeapElement::Hole; N],
            len: 0,
            hint_idx: 0,
            sort_on,
            seq: 0
        }
    }

    pub fn push(&mut self, frame: Frame<MTU>, marker: M) -> Result<usize, Frame<MTU>> {
        let heap_element = HeapElement::Filled(frame, self.seq, marker);
        for elem in self.data.iter_mut() {
            if *elem == HeapElement::Hole {
                *elem = heap_element;
                break;
            }
        }
        self.seq = self.seq.wrapping_add(1);
        if self.sort_on == SortOn::Push {
            self.data.sort_unstable();
            self.hint_idx = 0;
        }
        self.len += 1;

        Ok(0)
    }

    pub fn pop(&mut self) -> Option<Frame<MTU>> {
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
            HeapElement::Filled(frame, _, _) => {
                self.data[self.hint_idx] = HeapElement::Hole;
                self.hint_idx += 1;
                self.len -= 1;
                Some(frame)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameId;

    #[test]
    fn check_sort_by_seq() {
        let mut heap = Heap::<(), 8, 32>::new(SortOn::Push);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);

        assert_eq!(heap.pop().unwrap().data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);

        let mut heap = Heap::<(), 8, 32>::new(SortOn::Pop);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);

        assert_eq!(heap.pop().unwrap().data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn check_sort_by_id_and_seq() {
        let mut heap = Heap::<(), 8, 32>::new(SortOn::Push);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[1, 2, 3]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x1).unwrap(), &[4, 5, 6]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.push(Frame::new(FrameId::new_extended(0x123).unwrap(), &[7, 8, 9]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.push(Frame::new(FrameId::new_standard(0x1).unwrap(), &[1, 1]).unwrap(), ()), Ok(0));
        assert_eq!(heap.len(), 4);

        assert_eq!(heap.pop().unwrap().data(), &[1, 1]);
        assert_eq!(heap.len(), 3);
        assert_eq!(heap.pop().unwrap().data(), &[4, 5, 6]);
        assert_eq!(heap.len(), 2);
        assert_eq!(heap.pop().unwrap().data(), &[1, 2, 3]);
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.pop().unwrap().data(), &[7, 8, 9]);
        assert_eq!(heap.len(), 0);
        assert_eq!(heap.pop(), None);
    }
}