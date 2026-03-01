//! Types for serializing `Vec<T>` types.
//!
//! This module provides two types for serializing a `Vec<T>`: `TrailingVec` and a
//! set of `PrefixedVec`s with different length prefix types.
//!
//! `TrailingVec` is serialized without a length prefix, while the `PrefixedVec`s
//! are serialized with a length prefix determined by a type. The length prefix is useful
//! for deserializing vectors that are not the last field of a struct, as it allows the
//! deserializer to know how many bytes to read for the vector, while allowing for more
//! efficient storage depending on the expected length of the vector.
//!
//! The types in this module also implement the `Deref` trait, allowing them to be used
//! as regular `Vec<T>` in most contexts.

#[cfg(feature = "borsh")]
use borsh::{
    io::{ErrorKind, Read, Write},
    BorshDeserialize, BorshSerialize,
};
use {
    alloc::vec::Vec,
    core::{
        fmt::{Debug, Formatter},
        ops::Deref,
    },
};
#[cfg(feature = "wincode")]
use {
    core::mem::MaybeUninit,
    wincode::{
        config::ConfigCore,
        error::{write_length_encoding_overflow, ReadError},
        io::{Reader, Writer},
        ReadResult, SchemaRead, SchemaWrite, WriteResult,
    },
};

/// A `Vec<T>` serialized without a length prefix.
///
/// This is useful for serializing a `Vec<T>` that is the last field
/// of a struct, where the length can be inferred from the remaining
/// bytes.
///
/// Note that this type is not suitable for serializing `Vec`s that
/// are not the last field of a struct, as it will consume all
/// remaining bytes.
///
/// # Examples
///
/// Using `TrailingVec` in a struct results in the vector being
/// serialized without a length prefix.
///
/// ```
/// use spl_collections::TrailingVec;
/// use wincode::{SchemaRead, SchemaWrite};
///
/// #[derive(SchemaRead, SchemaWrite)]
/// pub struct MyStruct {
///   pub amount: u64,
///   pub items: TrailingVec<u32>,
/// }
///
/// let my_struct = MyStruct {
///   amount: 1_000_000_000,
///   items: TrailingVec::from(vec![1, 2, 3, 4, 5]),
/// };
///
/// let bytes = wincode::serialize(&my_struct).unwrap();
/// // Expected size:
/// //   - amount (8 bytes)
/// //   - items (remaining `Vec<T>` without a length prefix)
/// assert_eq!(bytes.len(), 8 + my_struct.items.len() * size_of::<u32>());
/// # let deserialized = wincode::deserialize::<MyStruct>(&bytes).unwrap();
///
/// # assert_eq!(deserialized.amount, my_struct.amount);
/// # assert_eq!(deserialized.items, my_struct.items);
/// ```
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TrailingVec<T>(Vec<T>);

impl<T> From<Vec<T>> for TrailingVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}

impl<T: Clone> From<&[T]> for TrailingVec<T> {
    fn from(value: &[T]) -> Self {
        Self(Vec::from(value))
    }
}

impl<const N: usize, T: Clone> From<&[T; N]> for TrailingVec<T> {
    fn from(value: &[T; N]) -> Self {
        Self(Vec::from(value))
    }
}

impl<T> Deref for TrailingVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Debug> Debug for TrailingVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

#[cfg(feature = "borsh")]
impl<T: BorshSerialize> BorshSerialize for TrailingVec<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        // Serialized items without a length prefix.
        self.0.iter().try_for_each(|item| item.serialize(writer))
    }
}

#[cfg(feature = "borsh")]
impl<T: BorshDeserialize> BorshDeserialize for TrailingVec<T> {
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let mut items: Vec<T> = Vec::new();

        while let Ok(item) = T::deserialize_reader(reader) {
            items.push(item);
        }

        Ok(Self(items))
    }
}

#[cfg(feature = "wincode")]
unsafe impl<T, C> SchemaWrite<C> for TrailingVec<T>
where
    C: ConfigCore,
    T: SchemaWrite<C>,
{
    type Src = Self;

    #[inline(always)]
    fn size_of(src: &Self::Src) -> WriteResult<usize> {
        Ok(src.0.len() * size_of::<T>())
    }

    #[inline(always)]
    fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
        // SAFETY: Serializing a slice `[T]` without a length prefix.
        unsafe {
            writer
                .write_slice_t(src.0.as_slice())
                .map_err(wincode::WriteError::Io)
        }
    }
}

#[cfg(feature = "wincode")]
unsafe impl<'de, T, C> SchemaRead<'de, C> for TrailingVec<T>
where
    C: ConfigCore,
    T: SchemaRead<'de, C, Dst = T>,
{
    type Dst = Self;

    fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        let mut items = Vec::new();

        while let Ok(item) = T::get(&mut reader) {
            items.push(item);
        }

        dst.write(Self(items));

        Ok(())
    }
}

/// Macro defining a `PrefixedVec` type with a specified length prefix type.
macro_rules! prefixed_vec_type {
    ( $name:tt, $prefix_type:tt ) => {
        #[doc = concat!("A `Vec<T>` serialized with an `", stringify!($prefix_type), "` length prefix.")]
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct $name<T>(Vec<T>);

        impl<T> From<Vec<T>> for $name<T> {
            fn from(value: Vec<T>) -> Self {
                Self(value)
            }
        }

        impl<T: Clone> From<&[T]> for $name<T> {
            fn from(value: &[T]) -> Self {
                Self(Vec::from(value))
            }
        }

        impl<const N: usize, T: Clone> From<&[T; N]> for $name<T> {
            fn from(value: &[T; N]) -> Self {
                Self(Vec::from(value))
            }
        }

        impl<T> Deref for $name<T> {
            type Target = Vec<T>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: Debug> Debug for $name<T> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{:?}", self.0))
            }
        }

        #[cfg(feature = "borsh")]
        impl<T: BorshSerialize> BorshSerialize for $name<T> {
            fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
                BorshSerialize::serialize(
                    &$prefix_type::try_from(self.0.len()).map_err(|_| ErrorKind::InvalidData)?,
                    writer,
                )?;
                self.0.iter().try_for_each(|item| item.serialize(writer))
            }
        }

        #[cfg(feature = "borsh")]
        impl<T: BorshDeserialize> BorshDeserialize for $name<T> {
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
                let prefix = $prefix_type::deserialize_reader(reader)? as usize;
                let mut items: Vec<T> = Vec::with_capacity(prefix);

                while items.len() < prefix {
                    let Ok(item) = T::deserialize_reader(reader) else {
                        return Err(ErrorKind::InvalidData.into());
                    };

                    items.push(item);
                }

                Ok(Self(items))
            }
        }

        #[cfg(feature = "wincode")]
        unsafe impl<T, C> SchemaWrite<C> for $name<T>
        where
            C: ConfigCore,
            T: SchemaWrite<C>,
        {
            type Src = Self;

            #[inline(always)]
            fn size_of(src: &Self::Src) -> WriteResult<usize> {
                Ok(core::mem::size_of::<$prefix_type>() + size_of::<T>() * src.0.len())
            }

            #[inline(always)]
            fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
                <$prefix_type as SchemaWrite<C>>::write(
                    &mut writer,
                    &$prefix_type::try_from(src.0.len())
                        .map_err(|_| write_length_encoding_overflow(stringify!($prefix_type::MAX)))?,
                )?;
                // SAFETY: Serializing a slice `[T]`.
                unsafe {
                    writer
                        .write_slice_t(src.0.as_slice())
                        .map_err(wincode::WriteError::Io)
                }
            }
        }

        #[cfg(feature = "wincode")]
        unsafe impl<'de, T, C> SchemaRead<'de, C> for $name<T>
        where
            C: ConfigCore,
            T: SchemaRead<'de, C, Dst = T>,
        {
            type Dst = Self;

            fn read(
                mut reader: impl Reader<'de>,
                dst: &mut MaybeUninit<Self::Dst>,
            ) -> ReadResult<()> {
                let mut prefix = MaybeUninit::<$prefix_type>::uninit();
                <$prefix_type as SchemaRead<'de, C>>::read(&mut reader, &mut prefix)?;
                // SAFETY: We have just read the prefix from the reader, so it is initialized.
                let prefix = unsafe { prefix.assume_init() } as usize;

                let mut items = Vec::with_capacity(prefix);

                while items.len() < prefix {
                    let Ok(item) = T::get(&mut reader) else {
                        return Err(ReadError::Custom("failed to deserialize"));
                    };

                    items.push(item);
                }

                dst.write(Self(items));

                Ok(())
            }
        }
    };
}

// A `PrefixedVec` with a `u8` length prefix.
prefixed_vec_type!(U8PrefixedVec, u8);

// A `PrefixedVec` with a `u16` length prefix.
prefixed_vec_type!(U16PrefixedVec, u16);

// A `PrefixedVec` with a `u32` length prefix.
prefixed_vec_type!(U32PrefixedVec, u32);

// A `PrefixedVec` with a `u64` length prefix.
prefixed_vec_type!(U64PrefixedVec, u64);

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use core::mem::size_of;
    use wincode::WriteError;

    use super::*;

    #[test]
    fn trailing_vec_borsh_round_trip() {
        const VALUES: [u64; 5] = [255u64; 5];

        let original: TrailingVec<u64> = TrailingVec::from(&VALUES);
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; size_of::<u64>() * VALUES.len()];

        original.serialize(&mut bytes.as_mut_slice()).unwrap();

        let serialized = TrailingVec::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized.as_slice(), VALUES);
        assert_eq!(serialized, original);
    }

    #[test]
    fn trailing_vec_wincode_round_trip() {
        const VALUES: [u64; 5] = [255u64; 5];

        let original: TrailingVec<u64> = TrailingVec::from(&VALUES);
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; size_of::<u64>() * VALUES.len()];

        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<TrailingVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized.as_slice(), VALUES);
        assert_eq!(serialized, original);
    }

    #[test]
    fn prefixed_vec_borsh_round_trip() {
        const VALUES: [u64; 10] = [255u64; 10];

        // u8 length prefix
        let original = U8PrefixedVec::from(&VALUES);
        let bytes = borsh::to_vec(&original).unwrap();

        let serialized = U8PrefixedVec::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);

        // u16 length prefix
        let original = U16PrefixedVec::from(&VALUES);
        let bytes = borsh::to_vec(&original).unwrap();

        let serialized = U16PrefixedVec::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);

        // u64 length prefix
        let original = U64PrefixedVec::from(&VALUES);
        let bytes = borsh::to_vec(&original).unwrap();

        let serialized = U64PrefixedVec::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);
    }

    #[test]
    fn prefixed_vec_wincode_round_trip() {
        const VALUES: [u64; 10] = [255u64; 10];

        // u8 length prefix
        let original = U8PrefixedVec::from(&VALUES);
        let mut bytes = [0u8; size_of::<u8>() + size_of::<u64>() * VALUES.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<U8PrefixedVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);

        // u16 length prefix
        let original = U16PrefixedVec::from(&VALUES);
        let mut bytes = [0u8; size_of::<u16>() + size_of::<u64>() * VALUES.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<U16PrefixedVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);

        // u32 length prefix
        let original = U32PrefixedVec::from(&VALUES);
        let mut bytes = [0u8; size_of::<u32>() + size_of::<u64>() * VALUES.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<U32PrefixedVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);

        // u64 length prefix
        let original = U64PrefixedVec::from(&VALUES);
        let mut bytes = [0u8; size_of::<u64>() + size_of::<u64>() * VALUES.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<U64PrefixedVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
        assert_eq!(serialized.as_slice(), VALUES);
    }

    #[test]
    fn invalid_prefixed_value() {
        const VALUES: [u8; 256] = [255u8; 256];

        let original = U8PrefixedVec::from(&VALUES);

        // borsh
        let result = borsh::to_vec(&original);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidData);

        // wincode
        let result = wincode::serialize(&original);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WriteError::LengthEncodingOverflow(_)
        ));
    }

    #[test]
    fn prefixed_vec_borsh_with_remaining_bytes() {
        // Bytes representation for a `U8PrefixedVec<u64>` with 8 `u64` values
        // followed by 16 additional bytes.
        let mut bytes = [255u8; 81];
        bytes[0] = 8;

        let mut reader = bytes.as_slice();
        let serialized = U8PrefixedVec::<u64>::deserialize(&mut reader).unwrap();

        assert_eq!(serialized.len(), 8);
        assert_eq!(serialized.as_slice(), &[!(0u64); 8]);
    }

    #[test]
    fn prefixed_vec_wincode_with_remaining_bytes() {
        // Bytes representation for a `U8PrefixedVec<u64>` with 8 `u64` values
        // followed by 16 additional bytes.
        let mut bytes = [255u8; 81];
        bytes[0] = 8;

        let serialized = wincode::deserialize::<U8PrefixedVec<u64>>(&bytes).unwrap();

        assert_eq!(serialized.len(), 8);
        assert_eq!(serialized.as_slice(), &[!(0u64); 8]);
    }
}
