use borsh::maybestd::{io::Write, string::String};
use sp_std::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};
use sp_core::ed25519::{Public as Ed25519Public, Signature as Ed25519Signature};

use crate::clients::host_functions::HostFunctionsProvider;
use crate::Height;

#[derive(Debug)]
pub struct ConversionError(String);

#[derive(Debug, Clone, PartialEq, Eq, codec::Encode, codec::Decode)]
pub struct PublicKey(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, codec::Encode, codec::Decode)]
pub enum Signature {
    Ed25519(Ed25519Signature),
}

#[derive(
    Debug,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    codec::Encode,
    codec::Decode,
)]
pub struct CryptoHash(pub [u8; 32]);

impl Signature {
    const LEN: usize = 64;

    pub fn from_raw(raw: &[u8]) -> Self {
        Self::Ed25519(Ed25519Signature::from_raw(raw.try_into().unwrap()))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(inner) => &inner.0,
        }
    }

    // TODO: we might want to create a trait for signature verification
    // or integrate this into HostFunctions
    pub fn verify<T: HostFunctionsProvider>(
        &self,
        data: impl AsRef<[u8; 32]>,
        public_key: PublicKey,
    ) -> bool {
        match self {
            Self::Ed25519(signature) => T::ed25519_recover(signature.as_ref(), data.as_ref())
                .map(|key| &key == public_key.0.as_ref())
                .unwrap_or(false),
        }
    }
}

impl PublicKey {
    const LEN: usize = 32;

    pub fn from_raw(raw: &[u8]) -> Self {
        Self(raw.try_into().unwrap())
    }
}

impl TryFrom<&[u8]> for CryptoHash {
    type Error = ConversionError;
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.len() != 32 {
            return Err(ConversionError("wrong size".into()));
        }
        let inner: [u8; 32] = v.try_into().unwrap();
        Ok(CryptoHash(inner))
    }
}

impl AsRef<[u8]> for CryptoHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<&PublicKey> for Ed25519Public {
    fn from(pubkey: &PublicKey) -> Ed25519Public {
        Ed25519Public(pubkey.0)
    }
}

impl TryFrom<&[u8]> for PublicKey {
    type Error = ConversionError;
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        if v.len() != 32 {
            return Err(ConversionError("wrong size".into()));
        }
        let inner: [u8; 32] = v.try_into().unwrap();
        Ok(PublicKey(inner))
    }
}

pub type BlockHeight = u64;
pub type AccountId = String;
pub type Balance = u128;
pub type Gas = u64;

pub type MerkleHash = CryptoHash;

#[derive(Debug, Clone, BorshDeserialize)]
pub struct MerklePath(pub Vec<MerklePathItem>);

#[derive(Debug, Clone)]
pub struct LightClientBlockLiteView {
    pub prev_block_hash: CryptoHash,
    pub inner_rest_hash: CryptoHash,
    pub inner_lite: BlockHeaderInnerLiteView,
}

#[derive(
    Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, codec::Encode, codec::Decode,
)]
pub struct LightClientBlockView {
    pub prev_block_hash: CryptoHash,
    pub next_block_inner_hash: CryptoHash,
    pub inner_lite: BlockHeaderInnerLiteView,
    pub inner_rest_hash: CryptoHash,
    pub next_bps: Option<Vec<ValidatorStakeView>>,
    pub approvals_after_next: Vec<Option<Signature>>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, codec::Encode, codec::Decode,
)]
pub struct BlockHeaderInnerLiteView {
    pub height: BlockHeight,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outcome_root: CryptoHash,
    pub timestamp: u64,
    pub timestamp_nanosec: u64,
    pub next_bp_hash: CryptoHash,
    // lets assume that this is the merkle root of all blocks in this epoch, so far.
    pub block_merkle_root: CryptoHash,
}

/// For some reason, when calculating the hash of the current block
/// `timestamp_nanosec` is ignored
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct BlockHeaderInnerLiteViewFinal {
    pub height: BlockHeight,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outcome_root: CryptoHash,
    pub timestamp: u64,
    pub next_bp_hash: CryptoHash,
    pub block_merkle_root: CryptoHash,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum ApprovalInner {
    Endorsement(CryptoHash),
    Skip(BlockHeight),
}

#[derive(
    Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, codec::Encode, codec::Decode,
)]
pub enum ValidatorStakeView {
    V1(ValidatorStakeViewV1),
}

#[derive(
    Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, codec::Encode, codec::Decode,
)]
pub struct ValidatorStakeViewV1 {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub stake: Balance,
}

#[derive(Debug, Clone, BorshDeserialize)]
pub struct ExecutionOutcomeView {
    /// Logs from this transaction or receipt.
    pub logs: Vec<String>,
    /// Receipt IDs generated by this transaction or receipt.
    pub receipt_ids: Vec<CryptoHash>,
    /// The amount of the gas burnt by the given transaction or receipt.
    pub gas_burnt: Gas,
    /// The amount of tokens burnt corresponding to the burnt gas amount.
    /// This value doesn't always equal to the `gas_burnt` multiplied by the gas price, because
    /// the prepaid gas price might be lower than the actual gas price and it creates a deficit.
    pub tokens_burnt: u128,
    /// The id of the account on which the execution happens. For transaction this is signer_id,
    /// for receipt this is receiver_id.
    pub executor_id: AccountId,
    /// Execution status. Contains the result in case of successful execution.
    pub status: Vec<u8>, // NOTE(blas): no need to deserialize this one (in order to avoid having to define too many unnecessary structs)
}

#[derive(Debug, BorshDeserialize)]
pub struct OutcomeProof {
    /// this is the block merkle proof.
    pub proof: Vec<MerklePathItem>,
    /// this is the hash of the block.
    pub block_hash: CryptoHash,
    /// transaction hash
    pub id: CryptoHash,
    pub outcome: ExecutionOutcomeView,
    // TODO: where are the proofs for the block that this tx belongs
    // in the block_merkle_root of our light client.
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub enum Direction {
    Left,
    Right,
}

impl ValidatorStakeView {
    pub fn into_validator_stake(self) -> ValidatorStakeViewV1 {
        match self {
            Self::V1(inner) => inner,
        }
    }
}

#[cfg_attr(feature = "deepsize_feature", derive(deepsize::DeepSizeOf))]
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct MerklePathItem {
    pub hash: MerkleHash,
    pub direction: Direction,
}

impl BorshDeserialize for Signature {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        let _key_type: [u8; 1] = BorshDeserialize::deserialize(buf)?;
        let array: [u8; Self::LEN] = BorshDeserialize::deserialize(buf)?;
        Ok(Signature::Ed25519(Ed25519Signature::from_raw(array)))
    }
}

impl BorshSerialize for Signature {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), borsh::maybestd::io::Error> {
        match self {
            Signature::Ed25519(signature) => {
                BorshSerialize::serialize(&0u8, writer)?;
                writer.write_all(&signature.0)?;
            }
        }
        Ok(())
    }
}

impl BorshSerialize for PublicKey {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), borsh::maybestd::io::Error> {
        BorshSerialize::serialize(&0u8, writer)?;
        writer.write_all(&self.0)?;
        Ok(())
    }
}

impl BorshDeserialize for PublicKey {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, borsh::maybestd::io::Error> {
        let _key_type: [u8; 1] = BorshDeserialize::deserialize(buf)?;
        Ok(Self(BorshDeserialize::deserialize(buf)?))
    }
}

impl LightClientBlockView {
    pub fn get_height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: self.inner_lite.height,
        }
    }

    pub fn current_block_hash<H: HostFunctionsProvider>(&self) -> CryptoHash {
        current_block_hash::<H>(
            H::sha256_digest(self.inner_lite.try_to_vec().unwrap().as_ref())
                .as_slice()
                .try_into()
                .unwrap(),
            self.inner_rest_hash,
            self.prev_block_hash,
        )
    }
}

/// The hash of the block is:
/// ```ignore
/// sha256(concat(
///     sha256(concat(
///         sha256(borsh(inner_lite)),
///         sha256(borsh(inner_rest)) // we can use inner_rest_hash as well
///     )
/// ),
/// prev_hash
///))
/// ```
fn current_block_hash<H: HostFunctionsProvider>(
    inner_lite_hash: CryptoHash,
    inner_rest_hash: CryptoHash,
    prev_block_hash: CryptoHash,
) -> CryptoHash {
    H::sha256_digest(
        [
            H::sha256_digest(
                [inner_lite_hash.as_ref(), inner_rest_hash.as_ref()]
                    .concat()
                    .as_ref(),
            )
            .as_ref(),
            prev_block_hash.as_ref(),
        ]
        .concat()
        .as_ref(),
    )
    .as_slice()
    .try_into()
    .unwrap()
}
