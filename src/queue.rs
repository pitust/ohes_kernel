use alloc::{boxed::Box, vec, vec::Vec};

#[derive(Debug, Clone)]
pub struct ArrayQueue<T> {
    vec: Vec<Option<T>>,
    head: usize,
    tail: usize,
    pub count: usize,
}
impl<T> ArrayQueue<T> {
    pub fn push(&mut self, x: T) {
        if self.count == self.vec.len() {
            panic!("queue overflown")
        }
        self.vec[self.head] = Some(x);
        self.head += 1;
        if self.head as usize == self.vec.len() {
            self.head = 0;
        }
        self.count += 1;
    }
    pub fn pop(&mut self) -> &T {
        if self.tail == self.head {
            panic!("Bad read");
        }
        self.count -= 1;
        let val = self.vec[self.tail].as_ref().unwrap();
        self.tail += 1;
        if self.tail as usize == self.vec.len() {
            self.tail = 0;
        }
        val
    }
    pub fn new(sz: usize) -> ArrayQueue<T> {
        let mut q = ArrayQueue {
            vec: vec![],
            head: 0,
            tail: 0,
            count: 0,
        };
        for _i in 0..sz {
            q.vec.push(None)
        }
        q
    }
}
