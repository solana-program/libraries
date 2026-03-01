//! Types for serializing strings.
//!
//! This module provides two types for serializing strings: `TrailingStr` and a
//! set of `PrefixedStr`.
//!
//! `TrailingStr` is serialized without a length prefix, while the `PrefixedStr`s
//! are serialized with a length prefix determined by a type. The length prefix is useful
//! for deserializing strings that are not the last field of a struct, as it allows the
//! deserializer to know how many bytes to read for the string, while allowing for more
//! efficient storage depending on the expected length of the string.
//!
//! The types in this module also implement the `Deref` trait, allowing them to be used
//! as `&str` in most contexts.

#[cfg(feature = "borsh")]
use borsh::{
    io::{ErrorKind, Read},
    BorshDeserialize, BorshSerialize,
};
use {
    crate::{TrailingVec, U16PrefixedVec, U32PrefixedVec, U64PrefixedVec, U8PrefixedVec},
    alloc::string::String,
    core::{
        fmt::{Debug, Formatter},
        ops::Deref,
        str::from_utf8_unchecked,
    },
};
#[cfg(feature = "wincode")]
use {
    core::{mem::MaybeUninit, str::from_utf8},
    wincode::{
        config::{Config, ConfigCore},
        io::Reader,
        ReadError, ReadResult, SchemaRead, SchemaWrite, UninitBuilder,
    },
};

/// A `str` serialized without a length prefix.
///
/// This is useful for serializing strings that are the last field
/// of a struct, where the length can be inferred from the remaining
/// bytes.
///
/// Note that this type is not suitable for serializing strings that
/// are not the last field of a struct, as it will consume all
/// remaining bytes.
///
/// # Examples
///
/// Using `TrailingStr` in a struct results in the string being
/// serialized without a length prefix.
///
/// ```
/// use spl_collections::TrailingStr;
/// use wincode::{SchemaRead, SchemaWrite};
///
/// #[derive(SchemaRead, SchemaWrite)]
/// pub struct MyStruct {
///   pub state: u8,
///   pub amount: u64,
///   pub description: TrailingStr,
/// }
///
/// let my_struct = MyStruct {
///   state: 1,
///   amount: 1_000_000_000,
///   description: TrailingStr::from(
///     "The quick brown fox jumps over the lazy dog"
///   ),
/// };
///
/// let bytes = wincode::serialize(&my_struct).unwrap();
/// // Expected size:
/// //   - state (1 byte)
/// //   - amount (8 bytes)
/// //   - description (remaining bytes without a length prefix)
/// assert_eq!(bytes.len(), 1 + 8 + my_struct.description.len());
/// # let deserialized = wincode::deserialize::<MyStruct>(&bytes).unwrap();
///
/// # assert_eq!(deserialized.state, my_struct.state);
/// # assert_eq!(deserialized.amount, my_struct.amount);
/// # assert_eq!(deserialized.description, my_struct.description);
/// ```
#[cfg_attr(feature = "borsh", derive(BorshSerialize))]
#[cfg_attr(feature = "wincode", derive(SchemaWrite, UninitBuilder))]
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TrailingStr(TrailingVec<u8>);

impl From<String> for TrailingStr {
    fn from(value: String) -> Self {
        Self(TrailingVec::from(value.as_bytes()))
    }
}

impl From<&str> for TrailingStr {
    fn from(value: &str) -> Self {
        Self(TrailingVec::from(value.as_bytes()))
    }
}

impl Deref for TrailingStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The `TrailingStr` type is only constructed
        // from valid UTF-8 strings.
        unsafe { from_utf8_unchecked(&self.0) }
    }
}

impl Debug for TrailingStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.deref()))
    }
}

#[cfg(feature = "borsh")]
impl BorshDeserialize for TrailingStr {
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let container = TrailingVec::<u8>::deserialize_reader(reader)?;

        // Validate that we got valid UTF-8 bytes, as `TrailingStr` must
        // always be valid UTF-8.
        if from_utf8(&container).is_err() {
            return Err(ErrorKind::InvalidData.into());
        }

        Ok(Self(container))
    }
}

#[cfg(feature = "wincode")]
unsafe impl<'de, C: Config> SchemaRead<'de, C> for TrailingStr {
    type Dst = Self;

    fn read(reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        let mut builder = TrailingStrUninitBuilder::<C>::from_maybe_uninit_mut(dst);
        builder.read_0(reader)?;

        let container = unsafe { builder.uninit_0_mut().assume_init_ref() };

        // Validate that we got valid UTF-8 bytes, as `TrailingStr` must
        // always be valid UTF-8.
        if from_utf8(container).is_err() {
            return Err(ReadError::Custom("invalid UTF-8 bytes"));
        }

        builder.finish();

        Ok(())
    }
}

/// Macro defining a `PrefixedStr` type with a specified length prefix type.
macro_rules! prefixed_str_type {
    ( $name:tt, $container_type:tt, $prefix_type:tt ) => {
        #[doc = concat!("A `str` that is serialized with an `", stringify!($prefix_type), "` length prefix.")]
        #[cfg_attr(feature = "borsh", derive(BorshSerialize))]
        #[cfg_attr(feature = "wincode", derive(SchemaWrite))]
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct $name($container_type<u8>);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self($container_type::from(value.as_bytes()))
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self($container_type::from(value.as_bytes()))
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                // SAFETY: `*PrefixedStr` types are only constructed
                // from valid UTF-8 strings.
                unsafe { from_utf8_unchecked(&self.0) }
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{:?}", self.deref()))
            }
        }

        #[cfg(feature = "borsh")]
        impl BorshDeserialize for $name {
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
                let container = $container_type::<u8>::deserialize_reader(reader)?;

                // Validate that we got valid UTF-8 bytes, as `TrailingStr` must
                // always be valid UTF-8.
                if from_utf8(&container).is_err() {
                    return Err(ErrorKind::InvalidData.into());
                }

                Ok(Self(container))
            }
        }

        #[cfg(feature = "wincode")]
        unsafe impl<'de, C: ConfigCore> SchemaRead<'de, C> for $name {
            type Dst = Self;

            fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
                let container = <$container_type::<u8> as SchemaRead<C>>::get(&mut reader)?;

                // Validate that we got valid UTF-8 bytes, as `TrailingStr` must
                // always be valid UTF-8.
                if from_utf8(&container).is_err() {
                    return Err(ReadError::Custom("invalid UTF-8 bytes"));
                }

                dst.write(Self(container));

                Ok(())
            }
        }
    };
}

// A `PrefixedStr` with a `u8` length prefix.
prefixed_str_type!(U8PrefixedStr, U8PrefixedVec, u8);

// A `PrefixedStr` with a `u16` length prefix.
prefixed_str_type!(U16PrefixedStr, U16PrefixedVec, u16);

// A `PrefixedStr` with a `u32` length prefix.
prefixed_str_type!(U32PrefixedStr, U32PrefixedVec, u32);

// A `PrefixedStr` with a `u64` length prefix.
prefixed_str_type!(U64PrefixedStr, U64PrefixedVec, u64);

#[cfg(test)]
mod tests {
    use {
        alloc::vec::Vec,
        borsh::{io::ErrorKind, BorshDeserialize, BorshSerialize},
        core::mem::size_of,
        wincode::WriteError,
    };

    use super::*;

    #[test]
    fn trailing_str_borsh_round_trip() {
        const DATA: &str = "Trailing strings have many characters";

        let original: TrailingStr = TrailingStr::from(String::from(DATA));
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; DATA.len()];

        original.serialize(&mut bytes.as_mut_slice()).unwrap();

        let serialized = TrailingStr::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
    }

    #[test]
    fn trailing_str_wincode_round_trip() {
        const DATA: &str = "Trailing strings have many characters";

        let original: TrailingStr = TrailingStr::from(String::from(DATA));
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; DATA.len()];

        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<TrailingStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized.deref(), DATA);
        assert_eq!(serialized, original);
    }

    #[test]
    fn prefixed_str_borsh_round_trip() {
        const TEXT: &str = "Prefixed strings have many characters";

        // u8 length prefix + string bytes
        let original = U8PrefixedStr::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(bytes[0], TEXT.len() as u8);
        assert_eq!(&bytes[1..], TEXT.as_bytes());

        let string = U8PrefixedStr::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.deref(), TEXT);

        // u16 length prefix + string bytes
        let original = U16PrefixedStr::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u16::from_le_bytes(unsafe { *(bytes[0..2].as_ptr() as *const [u8; 2]) }),
            TEXT.len() as u16
        );
        assert_eq!(&bytes[2..], TEXT.as_bytes());

        let string = U16PrefixedStr::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.deref(), TEXT);

        // u32 length prefix + string bytes
        let original = U32PrefixedStr::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u32::from_le_bytes(unsafe { *(bytes[0..4].as_ptr() as *const [u8; 4]) }),
            TEXT.len() as u32
        );
        assert_eq!(&bytes[4..], TEXT.as_bytes());

        let string = U32PrefixedStr::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.deref(), TEXT);

        // u64 length prefix + string bytes
        let original = U64PrefixedStr::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u64::from_le_bytes(unsafe { *(bytes[0..8].as_ptr() as *const [u8; 8]) }),
            TEXT.len() as u64
        );
        assert_eq!(&bytes[8..], TEXT.as_bytes());

        let string = U64PrefixedStr::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.deref(), TEXT);
    }

    #[test]
    fn prefixed_str_wincode_round_trip() {
        const TEXT: &str = "Prefixed strings have many characters";

        // u8 length prefix + string bytes
        let original = U8PrefixedStr::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u8>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(bytes[0], TEXT.len() as u8);
        assert_eq!(&bytes[1..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U8PrefixedStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.deref(), TEXT);
        assert_eq!(serialized, original);

        // u16 length prefix + string bytes
        let original = U16PrefixedStr::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u16>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u16::from_le_bytes(unsafe { *(bytes[0..2].as_ptr() as *const [u8; 2]) }),
            TEXT.len() as u16
        );
        assert_eq!(&bytes[2..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U16PrefixedStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.deref(), TEXT);
        assert_eq!(serialized, original);

        // u32 length prefix + string bytes
        let original = U32PrefixedStr::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u32>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u32::from_le_bytes(unsafe { *(bytes[0..4].as_ptr() as *const [u8; 4]) }),
            TEXT.len() as u32
        );
        assert_eq!(&bytes[4..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U32PrefixedStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.deref(), TEXT);
        assert_eq!(serialized, original);

        // u64 length prefix + string bytes
        let original = U64PrefixedStr::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u64>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u64::from_le_bytes(unsafe { *(bytes[0..8].as_ptr() as *const [u8; 8]) }),
            TEXT.len() as u64
        );
        assert_eq!(&bytes[8..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U64PrefixedStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.deref(), TEXT);
        assert_eq!(serialized, original);
    }

    #[test]
    fn invalid_prefixed_value() {
        let large_text = "a".repeat(256);

        let original = U8PrefixedStr::from(large_text);

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
    fn prefixed_str_borsh_with_remaining_bytes() {
        let value = "⚙️ serialized data with extra bytes";
        let mut bytes = Vec::<u8>::new();

        bytes.push(value.len() as u8);
        bytes.extend_from_slice(value.as_bytes());
        // Extra bytes that should be ignored.
        bytes.extend_from_slice(&[255u8; 16]);

        let mut reader = bytes.as_slice();
        let serialized = U8PrefixedStr::deserialize(&mut reader).unwrap();

        assert_eq!(serialized.len(), value.len());
        assert_eq!(serialized.deref(), value);
    }

    #[test]
    fn prefixed_str_wincode_with_remaining_bytes() {
        let value = "⚙️ serialized data with extra bytes";

        let mut bytes = Vec::<u8>::new();
        bytes.push(value.len() as u8);
        bytes.extend_from_slice(value.as_bytes());
        // Extra bytes that should be ignored.
        bytes.extend_from_slice(&[255u8; 16]);

        let serialized = wincode::deserialize::<U8PrefixedStr>(&bytes).unwrap();

        assert_eq!(serialized.len(), value.len());
        assert_eq!(serialized.deref(), value);
    }

    #[test]
    fn invalid_utf8_borsh() {
        // prefix + 2 invalid UTF-8 bytes
        let bytes = [2u8, 255, 255];

        // For `TrailingStr`, skip the prefix byte and attempt to deserialize the remaining
        // bytes as UTF-8. Expect an error due to the invalid UTF-8 bytes.
        let mut reader = bytes[1..].as_ref();
        let maybe_deserialized = TrailingStr::deserialize(&mut reader);

        assert!(maybe_deserialized.is_err());

        // For `PrefixedStr`, read the length prefix and then read the specified number of
        // bytes as URF-8. Expect an error due to the invalid UTF-8 bytes.
        let mut reader = bytes.as_slice();
        let maybe_deserialized = U8PrefixedStr::deserialize(&mut reader);

        assert!(maybe_deserialized.is_err());
    }

    #[test]
    fn invalid_utf8_wincode() {
        // prefix + 2 invalid UTF-8 bytes
        let bytes = [2u8, 255, 255];

        // For `TrailingStr`, skip the prefix byte and attempt to deserialize the remaining
        // bytes as UTF-8. Expect an error due to the invalid UTF-8 bytes.
        let maybe_deserialized = wincode::deserialize::<TrailingStr>(&bytes[1..]);

        assert!(maybe_deserialized.is_err());

        // For `PrefixedStr`, read the length prefix and then read the specified number of
        // bytes as URF-8. Expect an error due to the invalid UTF-8 bytes.
        let maybe_deserialized = wincode::deserialize::<U8PrefixedStr>(&bytes);

        assert!(maybe_deserialized.is_err());
    }
}
