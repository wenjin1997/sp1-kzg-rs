# SP1 Project Template

This is a template for creating an end-to-end [SP1](https://github.com/succinctlabs/sp1) project
that can generate a proof of any RISC-V program.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install)

## Running the Project

There are 3 main ways to run this project: execute a program, generate a core proof, and
generate an EVM-compatible proof.

### Build the Program

The program is automatically built through `script/build.rs` when the script is built.

### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

This will execute the program and display the output.

Macbook Air M2 Output:

```sh
commitment: 0x93efc82d2017e9c57834a1246463e64774e56183bb247c8fc9dd98c56817e878d97b05f5c8d900acf1fbbbca6f146556
z: 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000
y: 0x0000000000000000000000000000000000000000000000000000000000000000
input_proof: 0x92c51ff81dd71dab71cefecd79e8274b4b7ba36a0f40e2dc086bc4061c7f63249877db23297212991fd63e07b7ebc348
Execute the program...
stdout: 🚀 KZG-RS 验证工具
stdout: ==================
stdout: 开始运行 KZG 证明验证测试...
Program executed successfully.
Execute time: 748.337959ms
commitment: 0x93efc82d2017e9c57834a1246463e64774e56183bb247c8fc9dd98c56817e878d97b05f5c8d900acf1fbbbca6f146556
z: 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000
y: 0x0000000000000000000000000000000000000000000000000000000000000000
proof: 0x92c51ff81dd71dab71cefecd79e8274b4b7ba36a0f40e2dc086bc4061c7f63249877db23297212991fd63e07b7ebc348
result: true
Verify the proof...
Verify kzg proof time: 2.556959ms
Values are correct!
Number of cycles: 9508114
```

### Generate an SP1 Core Proof

To generate an SP1 [core proof](https://docs.succinct.xyz/docs/sp1/generating-proofs/proof-types#core-default) for your program:

```sh
cd script
cargo run --release -- --prove
```

Macbook Air M2 Output:

```sh
commitment: 0x93efc82d2017e9c57834a1246463e64774e56183bb247c8fc9dd98c56817e878d97b05f5c8d900acf1fbbbca6f146556
z: 0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000
y: 0x0000000000000000000000000000000000000000000000000000000000000000
input_proof: 0x92c51ff81dd71dab71cefecd79e8274b4b7ba36a0f40e2dc086bc4061c7f63249877db23297212991fd63e07b7ebc348
Setup the program for proving...
Setup time: 755.706208ms
Generate the proof...
stdout: 🚀 KZG-RS 验证工具
stdout: ==================
stdout: 开始运行 KZG 证明验证测试...
stdout: 🚀 KZG-RS 验证工具
stdout: ==================
stdout: 开始运行 KZG 证明验证测试...
Generate proof time: 759.993709125s
Successfully generated proof!
Verify the proof...
Verify proof time: 8.48499425s
```

### Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 16GB RAM to generate a Groth16 or PLONK proof. View the [SP1 docs](https://docs.succinct.xyz/docs/sp1/getting-started/hardware-requirements#local-proving) for more information.

Generating a proof that is cheap to verify on the EVM (e.g. Groth16 or PLONK) is more intensive than generating a core proof.

To generate a Groth16 proof:

```sh
cd script
cargo run --release --bin evm -- --system groth16
```

To generate a PLONK proof:

```sh
cargo run --release --bin evm -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of SP1 proofs
inside Solidity.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command in `script`:

```sh
cargo run --release --bin vkey
```

## Using the Prover Network

We highly recommend using the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction) for any non-trivial programs or benchmarking purposes. For more information, see the [key setup guide](https://docs.succinct.xyz/docs/network/developers/key-setup) to get started.

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `NETWORK_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
SP1_PROVER=network NETWORK_PRIVATE_KEY=... cargo run --release --bin evm
```

## Cycle Counts in SP1

In my Macbook Air M2, the cycle counts are as follows:

| Test                                   | Cycle Count |
| -------------------------------------- | ----------- |
| Verify blob KZG proof                  |   |
| Verify blob KZG proof batch (10 blobs) |  |
| Evaluate polynomial in evaluation form |   |
| Compute challenge                      |   |
| Verify KZG proof                       | 9,508,114   |