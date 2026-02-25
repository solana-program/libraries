use {bytemuck::Pod, core::num::TryFromIntError};

/// Marker trait for converting to/from Pod `uint`'s and `usize`
pub trait PodLength: Pod + TryFrom<usize, Error = TryFromIntError> + Into<usize> {}

/// Blanket implementation to automatically implement `PodLength` for any type
/// that satisfies the required bounds.
impl<T> PodLength for T where T: Pod + TryFrom<usize, Error = TryFromIntError> + Into<usize> {}
