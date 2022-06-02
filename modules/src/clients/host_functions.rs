use crate::core::ics02_client::error::Error;
use crate::prelude::*;
use core::marker::PhantomData;
use sp_core::H256;

/// This trait captures all the functions that the host chain should provide for
/// crypto operations.
pub trait HostFunctionsProvider: Clone + Send + Sync {
    /// Keccak 256 hash function
    fn keccak_256(input: &[u8]) -> [u8; 32];

    /// Compressed Ecdsa public key recovery from a signature
    fn secp256k1_ecdsa_recover_compressed(
        signature: &[u8; 65],
        value: &[u8; 32],
    ) -> Option<Vec<u8>>;

    /// Recover the ED25519 pubkey that produced this signature
    fn ed25519_verify(signature: &[u8; 64], value: &[u8; 32], pubkey: &[u8]) -> bool;

    /// This function should verify membership in a trie proof using parity's sp-trie package
    /// with a BlakeTwo256 Hasher
    fn verify_membership_trie_proof(
        root: &H256,
        proof: &[Vec<u8>],
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Error>;

    /// This function should verify non membership in a trie proof using parity's sp-trie package
    /// with a BlakeTwo256 Hasher
    fn verify_non_membership_trie_proof(
        root: &H256,
        proof: &[Vec<u8>],
        key: &[u8],
    ) -> Result<(), Error>;

    /// Conduct a 256-bit Sha2 hash
    fn sha256_digest(data: &[u8]) -> [u8; 32];
}

/// This is a work around that allows us to have one super trait [`HostFunctionsProvider`]
/// that encapsulates all the needed host functions by different subsytems, and then
/// implement the needed traits through this wrapper.
#[derive(Clone)]
pub struct HostFunctionsManager<T: HostFunctionsProvider>(PhantomData<T>);

// implementation for beefy host functions
impl<T> beefy_client::traits::HostFunctions for HostFunctionsManager<T>
where
    T: HostFunctionsProvider,
{
    fn keccak_256(input: &[u8]) -> [u8; 32] {
        T::keccak_256(input)
    }

    fn secp256k1_ecdsa_recover_compressed(
        signature: &[u8; 65],
        value: &[u8; 32],
    ) -> Option<Vec<u8>> {
        T::secp256k1_ecdsa_recover_compressed(signature, value)
    }
}

impl<T> tendermint_light_client_verifier::host_functions::HostFunctionsProvider for HostFunctionsManager<T>
    where
        T: HostFunctionsProvider,
{
    fn sha2_256(preimage: &[u8]) -> [u8; 32] {
        todo!()
    }

    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool {
        todo!()
    }

    fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> bool {
        todo!()
    }
}
