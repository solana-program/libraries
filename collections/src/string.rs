//! Types for serializing strings types.
//!
//! This module provides two types for serializing strings: `TrailingString` and a
//! set of `PrefixedString`.
//!
//! `TrailingString` is serialized without a length prefix, while the `PrefixedString`s
//! are serialized with a length prefix determined by a type. The length prefix is useful
//! for deserializing strings that are not the last field of a struct, as it allows the
//! deserializer to know how many bytes to read for the string, while allowing for more
//! efficient storage depending on the expected length of the string.
//!
//! The types in this module also implement the `Deref` trait, allowing them to be used
//! as regular `String` in most contexts.

use {
    alloc::string::{String, ToString},
    core::{
        fmt::{Debug, Formatter},
        ops::Deref,
    },
};
#[cfg(feature = "borsh")]
use {
    alloc::vec,
    borsh::{
        io::{ErrorKind, Read, Write},
        BorshDeserialize, BorshSerialize,
    },
};
#[cfg(feature = "wincode")]
use {
    core::mem::MaybeUninit,
    wincode::{
        config::ConfigCore,
        error::{invalid_utf8_encoding, write_length_encoding_overflow},
        io::{Reader, Writer},
        ReadResult, SchemaRead, SchemaWrite, WriteResult,
    },
};

#[cfg(feature = "borsh")]
/// Size of the buffer used to read the string.
const BUFFER_SIZE: usize = 1024;

/// A `String` serialized without a length prefix.
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
/// Using `TrailingString` in a struct results in the string being
/// serialized without a length prefix.
///
/// ```
/// use spl_collections::TrailingString;
/// use wincode::{SchemaRead, SchemaWrite};
///
/// #[derive(SchemaRead, SchemaWrite)]
/// pub struct MyStruct {
///   pub state: u8,
///   pub amount: u64,
///   pub description: TrailingString,
/// }
///
/// let my_struct = MyStruct {
///   state: 1,
///   amount: 1_000_000_000,
///   description: TrailingString::from(
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
#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TrailingString(String);

impl From<String> for TrailingString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for TrailingString {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Deref for TrailingString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for TrailingString {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

#[cfg(feature = "borsh")]
impl BorshSerialize for TrailingString {
    fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        // Serialize the string bytes without a length prefix.
        writer.write_all(self.0.as_bytes())
    }
}

#[cfg(feature = "borsh")]
impl BorshDeserialize for TrailingString {
    fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
        // Read the string in chunks until we reach the end of the reader.
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut s = String::new();

        loop {
            let bytes_read = reader.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            s.push_str(
                core::str::from_utf8(&buffer[..bytes_read]).map_err(|_| ErrorKind::InvalidData)?,
            );
        }

        Ok(Self(s))
    }
}

#[cfg(feature = "wincode")]
unsafe impl<C: ConfigCore> SchemaWrite<C> for TrailingString {
    type Src = Self;

    #[inline(always)]
    fn size_of(src: &Self::Src) -> WriteResult<usize> {
        Ok(src.0.len())
    }

    #[inline(always)]
    fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
        // Serialize the string bytes without a length prefix.
        unsafe {
            writer
                .write_slice_t(src.0.as_bytes())
                .map_err(wincode::WriteError::Io)
        }
    }
}

#[cfg(feature = "wincode")]
unsafe impl<'de, C: ConfigCore> SchemaRead<'de, C> for TrailingString {
    type Dst = Self;

    fn read(mut reader: impl Reader<'de>, dst: &mut MaybeUninit<Self::Dst>) -> ReadResult<()> {
        let mut s = String::new();
        let mut bytes_read = 0;

        loop {
            // SAFETY: Move the reader by `bytes_read` from the previous iteration.
            unsafe { reader.consume_unchecked(bytes_read) };

            // Read the string in chunks until we reach the end of the reader.
            let bytes = reader.fill_buf(BUFFER_SIZE)?;

            if bytes.is_empty() {
                break;
            }

            s.push_str(core::str::from_utf8(bytes).map_err(invalid_utf8_encoding)?);
            bytes_read = bytes.len();
        }

        dst.write(Self(s));

        Ok(())
    }
}

/// Macro defining a `PrefixedStr` type with a specified length prefix type.
macro_rules! prefixed_str_type {
    ( $name:tt, $prefix_type:tt ) => {
        #[doc = concat!("A `String` that is serialized with an `", stringify!($prefix_type), "` length prefix.")]
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct $name(String);

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl Deref for $name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{:?}", self.0))
            }
        }

        #[cfg(feature = "borsh")]
        impl BorshSerialize for $name {
            fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
                BorshSerialize::serialize(
                    &$prefix_type::try_from(self.0.len()).map_err(|_| ErrorKind::InvalidData)?,
                    writer,
                )?;
                writer.write_all(self.0.as_bytes())
            }
        }

        #[cfg(feature = "borsh")]
        impl BorshDeserialize for $name {
            fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
                let prefix = $prefix_type::deserialize_reader(reader)?;

                let mut buffer = vec![0u8; prefix as usize];
                reader.read_exact(&mut buffer)?;

                Ok(Self::from(
                    String::from_utf8(buffer).map_err(|_| ErrorKind::InvalidData)?,
                ))
            }
        }

        #[cfg(feature = "wincode")]
        unsafe impl<C: ConfigCore> SchemaWrite<C> for $name {
            type Src = Self;

            #[inline(always)]
            fn size_of(src: &Self::Src) -> WriteResult<usize> {
                Ok(core::mem::size_of::<$prefix_type>() + src.0.len())
            }

            #[inline(always)]
            fn write(mut writer: impl Writer, src: &Self::Src) -> WriteResult<()> {
                <$prefix_type as SchemaWrite<C>>::write(
                    &mut writer,
                    &$prefix_type::try_from(src.0.len())
                        .map_err(|_| write_length_encoding_overflow(stringify!($prefix_type::MAX)))?,
                )?;
                // SAFETY: Serializing a slice of `[u8]`.
                unsafe {
                    writer
                        .write_slice_t(src.0.as_bytes())
                        .map_err(wincode::WriteError::Io)
                }
            }
        }

        #[cfg(feature = "wincode")]
        unsafe impl<'de, C: ConfigCore> SchemaRead<'de, C> for $name {
            type Dst = Self;

            fn read(
                mut reader: impl Reader<'de>,
                dst: &mut MaybeUninit<Self::Dst>,
            ) -> ReadResult<()> {
                // Read the length prefix first to determine how many bytes to read for the string.
                let mut prefix = MaybeUninit::<$prefix_type>::uninit();
                <$prefix_type as SchemaRead<'de, C>>::read(&mut reader, &mut prefix)?;
                // SAFETY: We have just read the prefix from the reader, so it is initialized.
                let prefix = unsafe { prefix.assume_init() } as usize;

                let bytes = reader.fill_exact(prefix)?;
                dst.write($name::from(
                    core::str::from_utf8(bytes).map_err(invalid_utf8_encoding)?,
                ));

                Ok(())
            }
        }
    };
}

// A `PrefixedString` with a `u8` length prefix.
prefixed_str_type!(U8PrefixedString, u8);

// A `PrefixedString` with a `u16` length prefix.
prefixed_str_type!(U16PrefixedString, u16);

// A `PrefixedString` with a `u32` length prefix.
prefixed_str_type!(U32PrefixedString, u32);

// A `PrefixedString` with a `u64` length prefix.
prefixed_str_type!(U64PrefixedString, u64);

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use core::mem::size_of;
    use wincode::WriteError;

    use super::*;

    #[test]
    fn trailing_str_borsh_round_trip() {
        const DATA: &str = "Trailing strings have many characters";

        let original: TrailingString = TrailingString::from(String::from(DATA));
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; DATA.len()];

        original.serialize(&mut bytes.as_mut_slice()).unwrap();

        let serialized = TrailingString::try_from_slice(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized, original);
    }

    #[test]
    fn trailing_str_wincode_round_trip() {
        const DATA: &str = "Trailing strings have many characters";

        let original: TrailingString = TrailingString::from(String::from(DATA));
        // No need to reserve space for a length prefix.
        let mut bytes = [0u8; DATA.len()];

        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        let serialized = wincode::deserialize::<TrailingString>(&bytes).unwrap();

        assert_eq!(serialized.len(), original.len());
        assert_eq!(serialized.as_str(), DATA);
        assert_eq!(serialized, original);
    }

    #[test]
    fn prefixed_str_borsh_round_trip() {
        const TEXT: &str = "Prefixed strings have many characters";

        // u8 length prefix + string bytes
        let original = U8PrefixedString::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(bytes[0], TEXT.len() as u8);
        assert_eq!(&bytes[1..], TEXT.as_bytes());

        let string = U8PrefixedString::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.as_str(), TEXT);

        // u16 length prefix + string bytes
        let original = U16PrefixedString::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u16::from_le_bytes(unsafe { *(bytes[0..2].as_ptr() as *const [u8; 2]) }),
            TEXT.len() as u16
        );
        assert_eq!(&bytes[2..], TEXT.as_bytes());

        let string = U16PrefixedString::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.as_str(), TEXT);

        // u32 length prefix + string bytes
        let original = U32PrefixedString::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u32::from_le_bytes(unsafe { *(bytes[0..4].as_ptr() as *const [u8; 4]) }),
            TEXT.len() as u32
        );
        assert_eq!(&bytes[4..], TEXT.as_bytes());

        let string = U32PrefixedString::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.as_str(), TEXT);

        // u64 length prefix + string bytes
        let original = U64PrefixedString::from(String::from(TEXT));
        let bytes = borsh::to_vec(&original).unwrap();

        assert_eq!(
            u64::from_le_bytes(unsafe { *(bytes[0..8].as_ptr() as *const [u8; 8]) }),
            TEXT.len() as u64
        );
        assert_eq!(&bytes[8..], TEXT.as_bytes());

        let string = U64PrefixedString::try_from_slice(&bytes).unwrap();

        assert_eq!(string.len(), TEXT.len());
        assert_eq!(string.as_str(), TEXT);
    }

    #[test]
    fn prefixed_str_wincode_round_trip() {
        const TEXT: &str = "Prefixed strings have many characters";

        // u8 length prefix + string bytes
        let original = U8PrefixedString::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u8>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(bytes[0], TEXT.len() as u8);
        assert_eq!(&bytes[1..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U8PrefixedString>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.as_str(), TEXT);
        assert_eq!(serialized, original);

        // u16 length prefix + string bytes
        let original = U16PrefixedString::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u16>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u16::from_le_bytes(unsafe { *(bytes[0..2].as_ptr() as *const [u8; 2]) }),
            TEXT.len() as u16
        );
        assert_eq!(&bytes[2..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U16PrefixedString>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.as_str(), TEXT);
        assert_eq!(serialized, original);

        // u32 length prefix + string bytes
        let original = U32PrefixedString::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u32>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u32::from_le_bytes(unsafe { *(bytes[0..4].as_ptr() as *const [u8; 4]) }),
            TEXT.len() as u32
        );
        assert_eq!(&bytes[4..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U32PrefixedString>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.as_str(), TEXT);
        assert_eq!(serialized, original);

        // u64 length prefix + string bytes
        let original = U64PrefixedString::from(String::from(TEXT));
        let mut bytes = [0u8; size_of::<u64>() + TEXT.len()];
        wincode::serialize_into(bytes.as_mut_slice(), &original).unwrap();

        assert_eq!(
            u64::from_le_bytes(unsafe { *(bytes[0..8].as_ptr() as *const [u8; 8]) }),
            TEXT.len() as u64
        );
        assert_eq!(&bytes[8..], TEXT.as_bytes());

        let serialized = wincode::deserialize::<U64PrefixedString>(&bytes).unwrap();

        assert_eq!(serialized.len(), TEXT.len());
        assert_eq!(serialized.as_str(), TEXT);
        assert_eq!(serialized, original);
    }

    #[test]
    fn invalid_prefixed_value() {
        let large_text = "a".repeat(256);

        let original = U8PrefixedString::from(large_text);

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
}
