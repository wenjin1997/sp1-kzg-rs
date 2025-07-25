use crate::enums::KzgError;
use crate::kzg_proof::safe_scalar_affine_from_bytes;
use crate::{BYTES_PER_BLOB, BYTES_PER_FIELD_ELEMENT};
use alloc::{string::ToString, vec::Vec};
use bls12_381::Scalar;

macro_rules! define_bytes_type {
    ($name:ident, $size:expr) => {
        #[cfg_attr(
            feature = "rkyv",
            derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
        )]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone)]
        pub struct $name(
            #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))] pub [u8; $size],
        );

        impl $name {
            pub fn from_slice(slice: &[u8]) -> Result<Self, KzgError> {
                if slice.len() != $size {
                    return Err(KzgError::InvalidBytesLength(
                        "Invalid slice length".to_string(),
                    ));
                }
                let mut bytes = [0u8; $size];
                bytes.copy_from_slice(slice);
                Ok($name(bytes))
            }

            pub fn as_slice(&self) -> &[u8] {
                &self.0
            }
        }

        impl From<$name> for [u8; $size] {
            fn from(value: $name) -> [u8; $size] {
                value.0
            }
        }
    };
}

define_bytes_type!(Bytes32, 32);
define_bytes_type!(Bytes48, 48);
define_bytes_type!(Blob, BYTES_PER_BLOB);

// 为 Bytes32 实现 to_bytes 方法
impl Bytes32 {
    /// 转换为 Vec<u8>，用于 ABI 编码 bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

// 为 Bytes48 实现 to_bytes 方法
impl Bytes48 {
    /// 转换为 Vec<u8>，用于 ABI 编码 bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Blob {
    pub fn as_polynomial(&self) -> Result<Vec<Scalar>, KzgError> {
        self.0
            .chunks(BYTES_PER_FIELD_ELEMENT)
            .map(|slice| {
                Bytes32::from_slice(slice).and_then(|bytes| safe_scalar_affine_from_bytes(&bytes))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bytes32() {
        let bytes = crate::dtypes::Bytes32::from_slice(&[0u8; 32]).unwrap();
        assert_eq!(bytes.0.len(), 32);
        // 测试 to_bytes
        let v = bytes.to_bytes();
        assert_eq!(v.len(), 32);
        assert!(v.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_bytes48() {
        let bytes = crate::dtypes::Bytes48::from_slice(&[0u8; 48]).unwrap();
        assert_eq!(bytes.0.len(), 48);
        // 测试 to_bytes
        let v = bytes.to_bytes();
        assert_eq!(v.len(), 48);
        assert!(v.iter().all(|&b| b == 0));
    }
}
