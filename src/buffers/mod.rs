mod finite;
mod ring;

pub use self::finite::*;
pub use self::ring::*;

/// An enumeration of buffer creation errors
#[derive(Debug, Clone, Copy)]
pub enum BufferError {
    /// The input buffer is empty, causing construction of some buffer types to
    /// fail
    EmptyInput,
    /// Shift operation failed for want of room to shift
    ShiftFailure,
}
