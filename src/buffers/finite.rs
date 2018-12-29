use crate::buffers::BufferError;
use crate::{Arbitrary, Unstructured};

/// A source of unstructured data with a finite size
///
/// This buffer is a finite source of unstructured data. Once the data is
/// exhausted it stays exhausted.
pub struct FiniteBuffer<'a> {
    buffer: &'a [u8],
    offset: usize,
    virtual_len: usize,
    container_size_limit: usize,
}

impl<'a> FiniteBuffer<'a> {
    /// Create a new FiniteBuffer
    ///
    /// If the passed `buffer` is shorter than max_len the total number of bytes
    /// will be the bytes available in `buffer`. If `buffer` is longer than
    /// `max_len` the buffer will be trimmed.
    pub fn new(buffer: &'a [u8]) -> Result<Self, BufferError> {
        Ok(FiniteBuffer {
            buffer: buffer,
            offset: 0,
            virtual_len: buffer.len(),
            container_size_limit: buffer.len(),
        })
    }

    /// Set the non-default container size limit
    pub fn container_size_limit(mut self, csl: usize) -> Self {
        self.container_size_limit = csl;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Potential errors of the [`FiniteBuffer`]
pub enum FBError {
    /// A request was made to fill a buffer, shift etc but there were
    /// insufficient bytes to accomplish this
    InsufficientBytes,
}

impl<'a> Unstructured for FiniteBuffer<'a> {
    type Error = FBError;

    fn fill_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        if self.virtual_len.saturating_sub(self.offset) >= buffer.len() {
            let max = self.offset + buffer.len();
            for (i, idx) in (self.offset..max).enumerate() {
                buffer[i] = self.buffer[idx];
            }
            self.offset = max;
            Ok(())
        } else {
            Err(FBError::InsufficientBytes)
        }
    }

    // NOTE(blt) I'm not sure if this is the right definition. I don't
    // understand the purpose of container_size.
    fn container_size(&mut self) -> Result<usize, Self::Error> {
        <usize as Arbitrary>::arbitrary(self).map(|x| x % self.container_size_limit)
    }

    fn reset(&mut self) {
        self.offset = 0;
    }

    fn shift_right(&mut self, total: usize) -> Result<(), Self::Error> {
        if self.virtual_len.saturating_sub(self.offset) < total {
            Err(FBError::InsufficientBytes)
        } else {
            self.offset += total;
            Ok(())
        }
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
    fn finite_buffer_fill_buffer() {
        let x = [1, 2, 3, 4];
        let mut rb = FiniteBuffer::new(&x).unwrap().container_size_limit(10);
        let mut z = [0; 2];
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 2]);
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [3, 4]);
        assert!(rb.fill_buffer(&mut z).is_err());
    }

    #[test]
    fn finite_buffer_fill_buffer_shift() {
        let x = [1, 2, 3, 4];
        let mut rb = FiniteBuffer::new(&x).unwrap().container_size_limit(10);
        let mut z = [0; 2];
        rb.shift_right(1).unwrap();
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [2, 3]);
        assert!(rb.fill_buffer(&mut z).is_err());
    }

    #[test]
    fn finite_buffer_fill_buffer_shift_failure() {
        let x = [1, 2, 3, 4];
        let mut rb = FiniteBuffer::new(&x).unwrap().container_size_limit(10);
        assert_eq!(Err(FBError::InsufficientBytes), rb.shift_right(x.len() + 1));
    }

    #[test]
    fn finite_buffer_fill_buffer_shrink() {
        let x = [1, 2, 3, 4];
        let mut rb = FiniteBuffer::new(&x).unwrap();
        let mut z = [0; 2];
        assert_eq!(2, rb.shrink());
        rb.fill_buffer(&mut z).unwrap();
        assert_eq!(z, [1, 2]);
        assert!(rb.fill_buffer(&mut z).is_err());
    }
}
