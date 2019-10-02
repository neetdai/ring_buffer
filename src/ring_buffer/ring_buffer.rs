use std::cell::UnsafeCell;
use std::cmp::min;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::ops::Fn;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::usize::MAX;

pub enum Error {
    CallBack,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Error::CallBack => write!(f, "not set callback function to accquire buffer"),
        }
    }
}

pub struct RingBuffer {
    inner: *mut u8,
    size: usize,
    lock: AtomicBool,
    write_position: AtomicUsize,
    read_position: AtomicUsize,
    callback: Option<Box<dyn Fn(Vec<u8>) + 'static>>,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            inner: {
                let mut buffer: Vec<u8> = vec![0; size];
                buffer.as_mut_ptr()
            },
            size: size,
            lock: AtomicBool::new(false),
            write_position: AtomicUsize::new(0),
            read_position: AtomicUsize::new(0),
            callback: None,
        }
    }

    #[inline]
    fn read_position_and_write_position(&self) -> (usize, usize) {
        (
            self.read_position.load(Ordering::Acquire),
            self.write_position.load(Ordering::Acquire),
        )
    }

    fn is_full(&self) -> bool {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();

        write_position >= self.size && write_position - self.size >= read_position
    }

    fn is_empty(&self) -> bool {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();
        let len: usize = self.size;

        (write_position % len) == (read_position % len)
    }

    fn avaliable_write_len(&self) -> usize {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();

        if self.size == 0 {
            return 0;
        }

        if self.size < read_position {
            return 0;
        }

        let len: usize = self.size;
        if write_position >= read_position {
            len - (write_position % len) + (read_position % len)
        } else {
            (read_position % len) - (write_position % len)
        }
    }

    fn avaliable_read_len(&self) -> usize {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();

        let len: usize = self.size;
        if (write_position % len) >= (read_position % len) {
            write_position - read_position
        } else {
            len - (read_position % len) + (write_position % len)
        }
    }

    pub fn len(&self) -> usize {
        self.avaliable_read_len()
    }

    pub fn write(&self, buf: &[u8]) -> usize {
        loop {
            if self.lock.compare_and_swap(false, true, Ordering::AcqRel) == false {
                if self.is_full() {
                    self.lock.store(false, Ordering::Release);
                    return 0;
                }

                let avaliable_write_len: usize = min(self.avaliable_write_len(), buf.len());
                let write_position: usize = self.write_position.load(Ordering::Acquire);
                let inner_len: usize = self.size;

                buf[0..avaliable_write_len]
                    .iter()
                    .enumerate()
                    .for_each(|(index, item)| {
                        let tmp: *mut u8 =
                            unsafe { self.inner.add((index + write_position) % inner_len) };
                        unsafe {
                            tmp.write(*item);
                        };
                        self.write_position.store(
                            (self.write_position.load(Ordering::Acquire) + 1) % MAX,
                            Ordering::Release,
                        );
                    });

                self.lock.store(false, Ordering::Release);
                return avaliable_write_len;
            }
        }
    }

    pub fn read_all(&self) -> Vec<u8> {
        loop {
            if self.lock.compare_and_swap(false, true, Ordering::AcqRel) == false {
                let avaliable_read_len: usize = self.avaliable_read_len();
                let read_position: usize = self.read_position.load(Ordering::Acquire);
                let mut result: Vec<u8> = Vec::with_capacity(avaliable_read_len);
                let inner_len: usize = self.size;

                (read_position..(read_position + avaliable_read_len)).for_each(|index| {
                    result.push(unsafe { self.inner.add((index) % inner_len) as u8 });
                    self.read_position.store(
                        (self.read_position.load(Ordering::Acquire) + 1) % MAX,
                        Ordering::Release,
                    );
                });

                return result;
            }
        }
    }

    pub fn set_callback<T>(&mut self, fnc: T)
    where
        T: Fn(Vec<u8>) + 'static,
    {
        self.callback = Some(Box::new(fnc));
    }

    pub fn callback_by_write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if self.callback.is_none() {
            Err(Error::CallBack)
        } else {
            let mut start: usize = 0;
            let length: usize = buf.len();

            loop {
                if start >= length {
                    break;
                }
                start += self.write(&buf[start..]);
                let result: Vec<u8> = self.read_all();
                if self.is_full() {
                    self.callback.as_mut().map(|fnc| {
                        fnc(result);
                    });
                }
            }

            Ok(())
        }
    }
}

impl Debug for RingBuffer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {

        f.debug_struct("RingBuffer")
            .field("inner", &self.inner)
            .field("write_position", &self.write_position.load(Ordering::Acquire))
            .field("read_position", &self.read_position.load(Ordering::Acquire))
            .field("size", &self.size)
            .finish()
    }
}
