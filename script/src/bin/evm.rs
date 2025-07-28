//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can have an
//! EVM-Compatible proof generated which can be verified on-chain.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system groth16
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release --bin evm -- --system plonk
//! ```

use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};
use kzg_rs::{
    dtypes::{Blob, Bytes32, Bytes48},
    KzgError, PublicValuesStruct,
};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const KZG_RS_ELF: &[u8] = include_elf!("kzg-rs-program");

/// The arguments for the EVM command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(
        long,
        default_value = "0x93efc82d2017e9c57834a1246463e64774e56183bb247c8fc9dd98c56817e878d97b05f5c8d900acf1fbbbca6f146556"
    )]
    commitment: String,
    #[arg(
        long,
        default_value = "0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000"
    )]
    z: String,
    #[arg(
        long,
        default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
    )]
    y: String,
    #[arg(
        long,
        default_value = "0x92c51ff81dd71dab71cefecd79e8274b4b7ba36a0f40e2dc086bc4061c7f63249877db23297212991fd63e07b7ebc348"
    )]
    input_proof: String,
    #[arg(long, value_enum, default_value = "plonk")]
    system: ProofSystem,
}

/// Enum representing the available proof systems
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

/// A fixture that can be used to test the verification of SP1 zkVM proofs inside Solidity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1KZGRSProofFixture {
    commitment: Bytes48,
    z: Bytes32,
    y: Bytes32,
    input_proof: Bytes48,
    result: bool,
    vkey: String,
    public_values: String,
    sp1_proof: String,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = EVMArgs::parse();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the program.
    let (pk, vk) = client.setup(KZG_RS_ELF);

    // Setup the inputs with fixed test data
    let mut stdin = SP1Stdin::new();
    stdin.write(&args.commitment);
    stdin.write(&args.z);
    stdin.write(&args.y);
    stdin.write(&args.input_proof);

    println!("commitment: {}", args.commitment);
    println!("z: {}", args.z);
    println!("y: {}", args.y);
    println!("input_proof: {}", args.input_proof);
    println!("Proof System: {:?}", args.system);

    // Generate the proof based on the selected proof system.
    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system);
}

/// Create a fixture for the given proof.
fn create_proof_fixture(
    sp1_proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    system: ProofSystem,
) {
    // Deserialize the public values.
    let bytes = sp1_proof.public_values.as_slice();
    let PublicValuesStruct {
        commitment,
        z,
        y,
        proof,
        result,
    } = PublicValuesStruct::abi_decode(bytes).unwrap();

    // Convert Bytes to the correct types
    let commitment = Bytes48::from_bytes_vec(commitment.to_vec()).unwrap();
    let z = Bytes32::from_bytes_vec(z.to_vec()).unwrap();
    let y = Bytes32::from_bytes_vec(y.to_vec()).unwrap();
    let input_proof = Bytes48::from_bytes_vec(proof.to_vec()).unwrap();

    // Create the testing fixture so we can test things end-to-end.
    let fixture = SP1KZGRSProofFixture {
        commitment,
        z,
        y,
        input_proof,
        result,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        sp1_proof: format!("0x{}", hex::encode(sp1_proof.bytes())),
    };

    // The verification key is used to verify that the proof corresponds to the execution of the
    // program on the given input.
    //
    // Note that the verification key stays the same regardless of the input.
    println!("Verification Key: {}", fixture.vkey);

    // The public values are the values which are publicly committed to by the zkVM.
    //
    // If you need to expose the inputs or outputs of your program, you should commit them in
    // the public values.
    println!("Public Values: {}", fixture.public_values);

    // The proof proves to the verifier that the program was executed with some inputs that led to
    // the give public values.
    println!("SP1 Proof Bytes: {:?}", fixture.sp1_proof);

    // Save the fixture to a file.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
