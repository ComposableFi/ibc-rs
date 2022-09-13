use crate::core::ics02_client::error::Error;
use crate::prelude::*;


/// This trait captures all the functions that the host chain should provide for
/// crypto operations.
pub trait HostFunctionsProvider: Clone + Send + Sync + Default {
    /// Keccak 256 hash function
    fn keccak_256(input: &[u8]) -> [u8; 32];

    /// Compressed Ecdsa public key recovery from a signature
    fn secp256k1_ecdsa_recover_compressed(
        signature: &[u8; 65],
        value: &[u8; 32],
    ) -> Option<Vec<u8>>;

    /// Recover the ED25519 pubkey that produced this signature, given a arbitrarily sized message
    fn ed25519_verify(signature: &[u8; 64], msg: &[u8], pubkey: &[u8]) -> bool;

    /// This function should verify membership in a trie proof using sp_state_machine's read_child_proof_check
    fn verify_membership_trie_proof(
        root: &[u8; 32],
        proof: &[Vec<u8>],
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Error>;

    /// This function should verify non membership in a trie proof using sp_state_machine's read_child_proof_check
    fn verify_non_membership_trie_proof(
        root: &[u8; 32],
        proof: &[Vec<u8>],
        key: &[u8],
    ) -> Result<(), Error>;

    /// This function should verify membership in a trie proof using parity's sp-trie package
    /// with a BlakeTwo256 Hasher
    fn verify_timestamp_extrinsic(
        root: &[u8; 32],
        proof: &[Vec<u8>],
        value: &[u8],
    ) -> Result<(), Error>;

    /// Conduct a 256-bit Sha2 hash
    fn sha256_digest(data: &[u8]) -> [u8; 32];

    /// The SHA-256 hash algorithm
    fn sha2_256(message: &[u8]) -> [u8; 32];

    /// The SHA-512 hash algorithm
    fn sha2_512(message: &[u8]) -> [u8; 64];

    /// The SHA-512 hash algorithm with its output truncated to 256 bits.
    fn sha2_512_truncated(message: &[u8]) -> [u8; 32];

    /// SHA-3-512 hash function.
    fn sha3_512(message: &[u8]) -> [u8; 64];

    /// Ripemd160 hash function.
    fn ripemd160(message: &[u8]) -> [u8; 20];
}
