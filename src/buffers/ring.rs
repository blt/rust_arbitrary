use crate::buffers::BufferError;
use crate::{Arbitrary, Unstructured};

/// A source of unstructured data which returns the same data over and over again
///
/// This buffer acts as a ring buffer over the source of unstructured data,
/// allowing for an infinite amount of not-very-random data.
pub struct RingBuffer<'a> {
    buffer: &'a [u8],
    offset: usize,
    virtual_len: usize,
    container_size_limit: usize,
}

impl<'a> RingBuffer<'a> {
    /// Create a new RingBuffer
    pub fn new(buffer: &'a [u8]) -> Result<Self, BufferError> {
        if buffer.is_empty() {
            return Err(BufferError::EmptyInput);
        }
        Ok(RingBuffer {
            buffer,
            virtual_len: buffer.len(),
            offset: 0,
            container_size_limit: buffer.len(),
        })
    }

    /// Set the non-default container size limit
    pub fn container_size_limit(mut self, csl: usize) -> Self {
        self.container_size_limit = csl;
        self
    }
}

impl<'a> Unstructured for RingBuffer<'a> {
    type Error = ();
    fn fill_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let b = [
            &self.buffer[self.offset..self.virtual_len],
            &self.buffer[..self.offset],
        ];
        let it = ::std::iter::repeat(&b[..]).flat_map(|x| x).flat_map(|&x| x);
        self.offset = (self.offset + buffer.len()) % self.virtual_len;
        for (d, f) in buffer.iter_mut().zip(it) {
            *d = *f;
        }
        Ok(())
    }

    fn container_size(&mut self) -> Result<usize, Self::Error> {
        <usize as Arbitrary>::arbitrary(self).map(|x| x % self.container_size_limit)
    }

    fn reset(&mut self) {
        self.offset = 0;
        self.virtual_len = self.buffer.len();
    }

    fn shift_right(&mut self, total: usize) -> Result<(), Self::Error> {
        self.offset = (self.offset + total) % self.virtual_len;
        Ok(())
    }

    fn shrink(&mut self) -> usize {
        self.virtual_len /= 2;
        self.virtual_len
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ring_buffer_fill_buffer() {
        let x = [1, 2, 3, 4];
        let mut rb = RingBuffer::new(&x).unwrap();
        let mut z = [0; 10];
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 2, 3, 4, 1, 2, 3, 4, 1, 2]);
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [3, 4, 1, 2, 3, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn ring_buffer_fill_buffer_shrink() {
        let x = [1, 2, 3, 4];
        let mut rb = RingBuffer::new(&x).unwrap();
        let mut z = [0; 10];
        assert_eq!(2, rb.shrink());
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 2, 1, 2, 1, 2, 1, 2, 1, 2]);
        assert_eq!(1, rb.shrink());
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn ring_buffer_fill_buffer_shift() {
        let x = [1, 2, 3, 4];
        let mut rb = RingBuffer::new(&x).unwrap();
        let mut z = [0; 10];
        rb.shift_right(1).unwrap();
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [2, 3, 4, 1, 2, 3, 4, 1, 2, 3]);
        rb.shift_right(1).unwrap();
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 2, 3, 4, 1, 2, 3, 4, 1, 2]);
    }

    #[test]
    fn ring_buffer_container_size() {
        let x = [1, 2, 3, 4, 5];
        let mut rb = RingBuffer::new(&x).unwrap().container_size_limit(11);
        assert_eq!(rb.container_size().unwrap(), 9);
        assert_eq!(rb.container_size().unwrap(), 1);
        assert_eq!(rb.container_size().unwrap(), 2);
        assert_eq!(rb.container_size().unwrap(), 6);
        assert_eq!(rb.container_size().unwrap(), 1);
    }
}
