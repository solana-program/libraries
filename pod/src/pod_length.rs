use {
    crate::{
        error::PodSliceError,
        primitives::{PodU16, PodU32, PodU64},
    },
    bytemuck::Pod,
};

/// Marker trait for converting to/from Pod `uint`'s and `usize`
pub trait PodLength: Pod + Into<usize> + TryFrom<usize, Error = PodSliceError> {}

impl<T> PodLength for T where T: Pod + Into<usize> + TryFrom<usize, Error = PodSliceError> {}

impl TryFrom<usize> for PodU16 {
    type Error = PodSliceError;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        Ok(u16::try_from(val)?.into())
    }
}

impl From<PodU16> for usize {
    fn from(pod: PodU16) -> Self {
        u16::from(pod) as usize
    }
}

impl TryFrom<usize> for PodU32 {
    type Error = PodSliceError;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        Ok(u32::try_from(val)?.into())
    }
}

impl From<PodU32> for usize {
    fn from(pod: PodU32) -> Self {
        u32::from(pod) as usize
    }
}

impl TryFrom<usize> for PodU64 {
    type Error = PodSliceError;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        Ok(u64::try_from(val)?.into())
    }
}

impl From<PodU64> for usize {
    fn from(pod: PodU64) -> Self {
        u64::from(pod) as usize
    }
}
