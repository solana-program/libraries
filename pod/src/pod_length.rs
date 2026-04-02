use {crate::error::PodSliceError, bytemuck::Pod};

/// Marker trait for converting to/from Pod `uint`'s and `usize`
pub trait PodLength: Pod + Into<usize> + TryFrom<usize> {}

/// Blanket implementation to automatically implement `PodLength` for any type
/// that satisfies the required bounds.
impl<T> PodLength for T
where
    T: Pod + Into<usize> + TryFrom<usize>,
    PodSliceError: From<<T as TryFrom<usize>>::Error>,
{
}
