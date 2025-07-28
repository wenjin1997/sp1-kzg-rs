//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_sol_types::SolType;
use clap::Parser;
use kzg_rs::{
    dtypes::{Bytes32, Bytes48},
    kzg_proof::KzgProof,
    PublicValuesStruct,
};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const KZG_RS_ELF: &[u8] = include_elf!("kzg-rs-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

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
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&args.commitment);
    stdin.write(&args.z);
    stdin.write(&args.y);
    stdin.write(&args.input_proof);

    println!("commitment: {}", args.commitment);
    println!("z: {}", args.z);
    println!("y: {}", args.y);
    println!("input_proof: {}", args.input_proof);

    if args.execute {
        // Execute the program
        let (output, report) = client.execute(KZG_RS_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");

        let kzg_settings = match kzg_rs::KzgSettings::load_trusted_setup_file() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("加载可信设置失败: {:?}", e);
                return;
            }
        };

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice()).unwrap();
        let PublicValuesStruct {
            commitment,
            z,
            y,
            proof,
            result,
        } = decoded;

        println!("commitment: {}", commitment);
        println!("z: {}", z);
        println!("y: {}", y);
        println!("proof: {}", proof);
        println!("result: {}", result);

        let commitment = Bytes48::from_bytes_vec(commitment.to_vec()).unwrap();
        let z = Bytes32::from_bytes_vec(z.to_vec()).unwrap();
        let y = Bytes32::from_bytes_vec(y.to_vec()).unwrap();
        let proof = Bytes48::from_bytes_vec(proof.to_vec()).unwrap();

        let expected_result =
            KzgProof::verify_kzg_proof(&commitment, &z, &y, &proof, &kzg_settings);

        assert_eq!(true, expected_result.unwrap());
        println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(KZG_RS_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
