//
// sensors.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 14 2023
//

use std::collections::{VecDeque, vec_deque::Iter};

/// Ring buffer for sensor data
pub struct SensorBuffer<T> {
    inner: VecDeque<T>,
}

impl<T> SensorBuffer<T> {
    pub fn new(size: usize) -> Self {
        SensorBuffer { inner: VecDeque::with_capacity(size) }
    }

    pub fn push(&mut self, item: T) {
        if self.inner.len() == self.inner.capacity() {
            self.inner.pop_front();
            self.inner.push_back(item);
        }
        else {
            self.inner.push_back(item);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn iter(&self) -> Iter<T> {
        self.inner.iter()
    }
}
