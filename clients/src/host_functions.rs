use core::marker::PhantomData;
use ibc::host_functions::HostFunctionsProvider;
use ibc::prelude::*;

// implementation for beefy host functions
#[cfg(any(test, feature = "mocks", feature = "ics11_beefy"))]
impl<T> beefy_client_primitives::HostFunctions for HostFunctionsManager<T>
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

    fn verify_timestamp_extrinsic(
        root: sp_core::H256,
        proof: &[Vec<u8>],
        value: &[u8],
    ) -> Result<(), beefy_client_primitives::error::BeefyClientError> {
        T::verify_timestamp_extrinsic(root.as_fixed_bytes(), proof, value)
            .map_err(|_| From::from("Timestamp verification failed".to_string()))
    }
}

// implementation for tendermint functions
impl<T> tendermint_light_client_verifier::host_functions::HostFunctionsProvider
    for HostFunctionsManager<T>
where
    T: HostFunctionsProvider,
{
    fn sha2_256(preimage: &[u8]) -> [u8; 32] {
        T::sha256_digest(preimage)
    }

    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool {
        let mut signature = [0u8; 64];
        signature.copy_from_slice(sig);
        T::ed25519_verify(&signature, msg, pub_key)
    }

    fn secp256k1_verify(_sig: &[u8], _message: &[u8], _public: &[u8]) -> bool {
        unimplemented!()
    }
}

/// This is a work around that allows us to have one super trait [`HostFunctionsProvider`]
/// that encapsulates all the needed host functions by different subsytems, and then
/// implement the needed traits through this wrapper.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct HostFunctionsManager<T: HostFunctionsProvider>(PhantomData<T>);

// implementation for ics23
impl<H: HostFunctionsProvider> ics23::HostFunctionsProvider for HostFunctionsManager<H> {
    fn sha2_256(message: &[u8]) -> [u8; 32] {
        H::sha2_256(message)
    }

    fn sha2_512(message: &[u8]) -> [u8; 64] {
        H::sha2_512(message)
    }

    fn sha2_512_truncated(message: &[u8]) -> [u8; 32] {
        H::sha2_512_truncated(message)
    }

    fn sha3_512(message: &[u8]) -> [u8; 64] {
        H::sha3_512(message)
    }

    fn ripemd160(message: &[u8]) -> [u8; 20] {
        H::ripemd160(message)
    }
}
