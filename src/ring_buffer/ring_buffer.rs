use std::cmp::min;
use std::fmt::{Debug, Error as FmtError, Formatter};
use std::ops::Fn;
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
    inner: Vec<u8>,
    write_position: usize,
    read_position: usize,
    callback: Option<Box<dyn Fn(Vec<u8>) + 'static>>,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            inner: vec![0; size],
            write_position: 0,
            read_position: 0,
            callback: None,
        }
    }

    #[inline]
    fn read_position_and_write_position(&self) -> (usize, usize) {
        (self.read_position, self.write_position)
    }

    fn is_full(&self) -> bool {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();

        write_position >= self.inner.len() && write_position - self.inner.len() >= read_position
    }

    fn is_empty(&self) -> bool {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();
        let len: usize = self.inner.len();

        (write_position % len) == (read_position % len)
    }

    fn avaliable_write_len(&self) -> usize {
        let read_position: usize = self.read_position;
        let write_position: usize = self.write_position;

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
            (read_position % len) - (write_position % len)
        }
    }

    fn avaliable_read_len(&self) -> usize {
        let (read_position, write_position): (usize, usize) =
            self.read_position_and_write_position();

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
        if self.is_full() {
            return 0;
        }

        let avaliable_write_len: usize = min(self.avaliable_write_len(), buf.len());
        let write_position: usize = self.write_position;
        let inner_len: usize = self.inner.len();

        buf[0..avaliable_write_len]
            .iter()
            .enumerate()
            .for_each(|(index, item)| {
                self.inner[(index + write_position) % inner_len] = *item;
                self.write_position = (self.write_position + 1) % MAX;
            });

        avaliable_write_len
    }

    pub fn read_all(&mut self) -> Vec<u8> {
        let avaliable_read_len: usize = self.avaliable_read_len();
        let read_position: usize = self.read_position;
        let mut result: Vec<u8> = Vec::with_capacity(avaliable_read_len);
        let inner_len: usize = self.inner.len();

        (read_position..(read_position + avaliable_read_len)).for_each(|index| {
            result.push(self.inner[(index) % inner_len]);
            self.read_position = (self.read_position + 1) % MAX;
        });

        result
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
            .field("write_position", &self.write_position)
            .field("read_position", &self.read_position)
            .finish()
    }
}
