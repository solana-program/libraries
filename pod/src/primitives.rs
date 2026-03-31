//! Primitive types that can be used in `Pod`s.
//!
//! These are re-exported from [`solana_zero_copy::unaligned`].
#[cfg(not(target_arch = "bpf"))]
pub use solana_zero_copy::unaligned::U128 as PodU128;
pub use solana_zero_copy::unaligned::{
    Bool as PodBool, I16 as PodI16, I64 as PodI64, U16 as PodU16, U32 as PodU32, U64 as PodU64,
};

#[cfg(test)]
mod tests {
    use {super::*, crate::bytemuck::pod_from_bytes};

    #[test]
    fn test_pod_bool() {
        assert!(pod_from_bytes::<PodBool>(&[]).is_err());
        assert!(pod_from_bytes::<PodBool>(&[0, 0]).is_err());

        for i in 0..=u8::MAX {
            assert_eq!(i != 0, bool::from(pod_from_bytes::<PodBool>(&[i]).unwrap()));
        }
    }

    #[cfg(feature = "serde-traits")]
    #[test]
    fn test_pod_bool_serde() {
        let pod_false: PodBool = false.into();
        let pod_true: PodBool = true.into();

        let serialized_false = serde_json::to_string(&pod_false).unwrap();
        let serialized_true = serde_json::to_string(&pod_true).unwrap();
        assert_eq!(&serialized_false, "false");
        assert_eq!(&serialized_true, "true");

        let deserialized_false = serde_json::from_str::<PodBool>(&serialized_false).unwrap();
        let deserialized_true = serde_json::from_str::<PodBool>(&serialized_true).unwrap();
        assert_eq!(pod_false, deserialized_false);
        assert_eq!(pod_true, deserialized_true);
    }

    #[test]
    fn test_pod_u16() {
        assert!(pod_from_bytes::<PodU16>(&[]).is_err());
        assert_eq!(1u16, u16::from(*pod_from_bytes::<PodU16>(&[1, 0]).unwrap()));
    }

    #[cfg(feature = "serde-traits")]
    #[test]
    fn test_pod_u16_serde() {
        let pod_u16: PodU16 = u16::MAX.into();

        let serialized = serde_json::to_string(&pod_u16).unwrap();
        assert_eq!(&serialized, "65535");

        let deserialized = serde_json::from_str::<PodU16>(&serialized).unwrap();
        assert_eq!(pod_u16, deserialized);
    }

    #[test]
    fn test_pod_i16() {
        assert!(pod_from_bytes::<PodI16>(&[]).is_err());
        assert_eq!(
            -1i16,
            i16::from(*pod_from_bytes::<PodI16>(&[255, 255]).unwrap())
        );
    }

    #[cfg(feature = "serde-traits")]
    #[test]
    fn test_pod_i16_serde() {
        let pod_i16: PodI16 = i16::MAX.into();

        println!("pod_i16 {:?}", pod_i16);

        let serialized = serde_json::to_string(&pod_i16).unwrap();
        assert_eq!(&serialized, "32767");

        let deserialized = serde_json::from_str::<PodI16>(&serialized).unwrap();
        assert_eq!(pod_i16, deserialized);
    }

    #[test]
    fn test_pod_u64() {
        assert!(pod_from_bytes::<PodU64>(&[]).is_err());
        assert_eq!(
            1u64,
            u64::from(*pod_from_bytes::<PodU64>(&[1, 0, 0, 0, 0, 0, 0, 0]).unwrap())
        );
    }

    #[cfg(feature = "serde-traits")]
    #[test]
    fn test_pod_u64_serde() {
        let pod_u64: PodU64 = u64::MAX.into();

        let serialized = serde_json::to_string(&pod_u64).unwrap();
        assert_eq!(&serialized, "18446744073709551615");

        let deserialized = serde_json::from_str::<PodU64>(&serialized).unwrap();
        assert_eq!(pod_u64, deserialized);
    }

    #[test]
    fn test_pod_i64() {
        assert!(pod_from_bytes::<PodI64>(&[]).is_err());
        assert_eq!(
            -1i64,
            i64::from(
                *pod_from_bytes::<PodI64>(&[255, 255, 255, 255, 255, 255, 255, 255]).unwrap()
            )
        );
    }

    #[cfg(feature = "serde-traits")]
    #[test]
    fn test_pod_i64_serde() {
        let pod_i64: PodI64 = i64::MAX.into();

        let serialized = serde_json::to_string(&pod_i64).unwrap();
        assert_eq!(&serialized, "9223372036854775807");

        let deserialized = serde_json::from_str::<PodI64>(&serialized).unwrap();
        assert_eq!(pod_i64, deserialized);
    }

    #[cfg(not(target_arch = "bpf"))]
    #[test]
    fn test_pod_u128() {
        assert!(pod_from_bytes::<PodU128>(&[]).is_err());
        assert_eq!(
            1u128,
            u128::from(
                *pod_from_bytes::<PodU128>(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                    .unwrap()
            )
        );
    }

    #[cfg(all(feature = "serde-traits", not(target_arch = "bpf")))]
    #[test]
    fn test_pod_u128_serde() {
        let pod_u128: PodU128 = u128::MAX.into();

        let serialized = serde_json::to_string(&pod_u128).unwrap();
        assert_eq!(&serialized, "340282366920938463463374607431768211455");

        let deserialized = serde_json::from_str::<PodU128>(&serialized).unwrap();
        assert_eq!(pod_u128, deserialized);
    }

    #[cfg(feature = "wincode")]
    mod wincode_tests {
        use {super::*, test_case::test_case};

        #[test_case(PodBool::from_bool(true))]
        #[test_case(PodBool::from_bool(false))]
        #[test_case(PodU16::from_primitive(u16::MAX))]
        #[test_case(PodI16::from_primitive(i16::MIN))]
        #[test_case(PodU32::from_primitive(u32::MAX))]
        #[test_case(PodU64::from_primitive(u64::MAX))]
        #[test_case(PodI64::from_primitive(i64::MIN))]
        #[cfg(not(target_arch = "bpf"))]
        #[test_case(PodU128::from_primitive(u128::MAX))]
        fn wincode_roundtrip<
            T: PartialEq
                + std::fmt::Debug
                + for<'de> wincode::SchemaRead<'de, wincode::config::DefaultConfig, Dst = T>
                + wincode::SchemaWrite<wincode::config::DefaultConfig, Src = T>,
        >(
            pod: T,
        ) {
            let bytes = wincode::serialize(&pod).unwrap();
            let deserialized: T = wincode::deserialize(&bytes).unwrap();
            assert_eq!(pod, deserialized);
        }
    }
}
