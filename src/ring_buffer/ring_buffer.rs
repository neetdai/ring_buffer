use std::cmp::min;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::usize::MAX;

#[derive(Debug)]
pub struct RingBuffer {
    inner: Vec<u8>,
    write_position: AtomicUsize,
    read_position: AtomicUsize,
    lock: AtomicBool,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            inner: vec![0; size],
            write_position: AtomicUsize::new(0),
            read_position: AtomicUsize::new(0),
            lock: AtomicBool::new(false),
        }
    }

    #[inline]
    fn read_position_and_write_position(&self) -> (usize, usize) {
        (self.read_position.load(Ordering::Acquire), self.write_position.load(Ordering::Acquire))
    }

    fn is_full(&self) -> bool {
        let (read_position, write_position): (usize, usize) = self.read_position_and_write_position();

        write_position >= self.inner.len() && write_position - self.inner.len() >= read_position
    }

    fn is_empty(&self) -> bool {
        let (read_position, write_position): (usize, usize) = self.read_position_and_write_position();
        let len: usize = self.inner.len();

        (write_position % len )== (read_position % len)
    }

    fn avaliable_write_len(&self) -> usize {
        let read_position: usize = self.read_position.load(Ordering::Acquire);
        let write_position: usize = self.write_position.load(Ordering::Acquire);

        if self.inner.is_empty() {
            return 0;
        }

        if self.inner.len() < read_position {
            return 0;
        }

        let len: usize = self.inner.len();
        if write_position >= read_position {
            len - (write_position % len) + (read_position % len)
        } else {
            (read_position % len)  - (write_position % len)
        }
    }

    fn avaliable_read_len(&self) -> usize {
        let (read_position, write_position): (usize, usize) = self.read_position_and_write_position();

        let len: usize = self.inner.len();
        if (write_position % len) >= (read_position % len) {
            write_position - read_position
        } else {
            len - (read_position % len) + (write_position % len)
        }
    }

    pub fn len(&self) -> usize {
        self.avaliable_read_len()
    }


    pub fn write(&mut self, buf: &[u8]) -> usize {
        loop {
            if !self.lock.compare_and_swap(false, true, Ordering::AcqRel) {
                if self.is_full() {
                    self.lock.store(false, Ordering::Release);
                    return 0;
                }

                let avaliable_write_len: usize = min(self.avaliable_write_len(),buf.len());
                let write_position: usize = self.write_position.load(Ordering::Acquire);
                let inner_len: usize = self.inner.len();

                buf[0..avaliable_write_len]
                    .iter()
                    .enumerate()
                    .for_each(|(index, item)| {
                        self.inner[(index + write_position) % inner_len] = *item;
                        // self.write_position.store((index + write_position + 1) % inner_len, Ordering::Release);
                        self.write_position.fetch_add(1, Ordering::Release);
                    });

                self.write_position.compare_and_swap(MAX, MAX % MAX, Ordering::AcqRel);

                self.lock.store(false, Ordering::Release);
                return avaliable_write_len;
            }
        }
    }

    pub fn read(&mut self) -> Vec<u8> {
        loop {
            if !self.lock.compare_and_swap(false, true, Ordering::AcqRel) {

                let avaliable_read_len: usize = self.avaliable_read_len();
                let read_position: usize = self.read_position.load(Ordering::Acquire);
                let mut result: Vec<u8> = Vec::with_capacity(avaliable_read_len);
                let inner_len: usize = self.inner.len();

                (read_position..(read_position + avaliable_read_len))
                    .for_each(|index| {
                        result.push(self.inner[(index) % inner_len]);
                        // self.read_position.store((index + 1) % inner_len, Ordering::Release);
                        self.read_position.fetch_add(1, Ordering::Release);
                    });

                self.read_position.compare_and_swap(MAX, MAX % MAX, Ordering::AcqRel);

                self.lock.store(false, Ordering::Release);
                return result;
            }
        }
    }
}
