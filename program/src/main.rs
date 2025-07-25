//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use kzg_rs::{
    dtypes::{Blob, Bytes32, Bytes48},
    enums::KzgError,
    kzg_proof::KzgProof,
    trusted_setup::KzgSettings,
    PublicValuesStruct,
};
use std::env;

pub fn main() {
    println!("🚀 KZG-RS 验证工具");
    println!("==================");

    println!("开始运行 KZG 证明验证测试...");

    let kzg_settings = match KzgSettings::load_trusted_setup_file() {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("加载可信设置失败: {:?}", e);
            return;
        }
    };

    let commitment = sp1_zkvm::io::read::<&str>();
    let z = sp1_zkvm::io::read::<&str>();
    let y = sp1_zkvm::io::read::<&str>();
    let proof = sp1_zkvm::io::read::<&str>();

    let input = Input {
        commitment,
        z,
        y,
        proof,
    };

    let test: Test<Input> = Test {
        input: input,
        output: Some(true),
    };

    let (Ok(commitment), Ok(z), Ok(y), Ok(proof)) = (
        test.input.get_commitment(),
        test.input.get_z(),
        test.input.get_y(),
        test.input.get_proof(),
    ) else {
        println!("✗ 测试失败：有效输入被正确接受");
        return;
    };

    let result = KzgProof::verify_kzg_proof(&commitment, &z, &y, &proof, &kzg_settings);

    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct {
        commitment: commitment.to_bytes(),
        z: z.to_bytes(),
        y: y.to_bytes(),
        proof: proof.to_bytes(),
        result: result.unwrap(),
    });
    sp1_zkvm::io::commit_slice(&bytes);
}

// 从测试模块中提取的结构体和函数
trait FromHex {
    fn from_hex(hex: &str) -> Result<Self, KzgError>
    where
        Self: Sized;
}

fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, KzgError> {
    let trimmed_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(trimmed_str)
        .map_err(|e| KzgError::InvalidHexFormat(format!("Failed to decode hex: {}", e)))
}

impl FromHex for Bytes48 {
    fn from_hex(hex_str: &str) -> Result<Self, KzgError> {
        Self::from_slice(&hex_to_bytes(hex_str).unwrap())
    }
}

impl FromHex for Bytes32 {
    fn from_hex(hex_str: &str) -> Result<Self, KzgError> {
        Self::from_slice(&hex_to_bytes(hex_str).unwrap())
    }
}

impl FromHex for Blob {
    fn from_hex(hex_str: &str) -> Result<Self, KzgError> {
        Self::from_slice(&hex_to_bytes(hex_str).unwrap())
    }
}

#[derive(Debug)]
pub struct Input<'a> {
    commitment: &'a str,
    z: &'a str,
    y: &'a str,
    proof: &'a str,
}

impl Input<'_> {
    pub fn get_commitment(&self) -> Result<Bytes48, KzgError> {
        Bytes48::from_hex(self.commitment)
    }

    pub fn get_z(&self) -> Result<Bytes32, KzgError> {
        Bytes32::from_hex(self.z)
    }

    pub fn get_y(&self) -> Result<Bytes32, KzgError> {
        Bytes32::from_hex(self.y)
    }

    pub fn get_proof(&self) -> Result<Bytes48, KzgError> {
        Bytes48::from_hex(self.proof)
    }
}

#[derive(Debug)]
pub struct Test<I> {
    pub input: I,
    output: Option<bool>,
}

impl<I> Test<I> {
    pub fn get_output(&self) -> Option<bool> {
        self.output
    }
}
