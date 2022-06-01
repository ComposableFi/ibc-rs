/// ClientState from Near tracks its head, {current, next} x {epoch, validators}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// LightClientBlockView contains most of the state needed to validate
    /// a future state transition
    #[prost(message, optional, tag="1")]
    pub head: ::core::option::Option<LightClientBlockView>,
    /// CyrptoHash representing the current epoch
    #[prost(bytes="vec", tag="2")]
    pub current_epoch: ::prost::alloc::vec::Vec<u8>,
    /// CyrptoHash representing the next epoch
    #[prost(bytes="vec", tag="3")]
    pub next_epoch: ::prost::alloc::vec::Vec<u8>,
    /// Tracks the set of validators that will vote on blocks in the current epoch
    #[prost(message, repeated, tag="4")]
    pub current_validators: ::prost::alloc::vec::Vec<ValidatorStakeView>,
    /// Tracks the set of validators that will vote on blocks in the next epoch
    #[prost(message, repeated, tag="5")]
    pub next_validators: ::prost::alloc::vec::Vec<ValidatorStakeView>,
}
/// LightClientBlockView contains most of the state needed to validate
/// a future state transition
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientBlockView {
    #[prost(bytes="vec", tag="1")]
    pub prev_block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub next_block_inner_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    pub inner_lite: ::core::option::Option<BlockHeaderInnerLiteView>,
    #[prost(bytes="vec", tag="4")]
    pub inner_rest_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="5")]
    pub next_bps: ::core::option::Option<ValidatorStakeView>,
    #[prost(message, repeated, tag="6")]
    pub approvals_after_next: ::prost::alloc::vec::Vec<MaybeSignature>,
}
/// BlockHeaderInnerLiteView for the current head (which contains height, epoch_id,
/// next_epoch_id, prev_state_root, outcome_root, timestamp, the hash of the block
/// producers set for the next epoch next_bp_hash, and the merkle root of all
/// the block hashes block_merkle_root);
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeaderInnerLiteView {
    #[prost(uint64, tag="1")]
    pub block_height: u64,
    #[prost(bytes="vec", tag="2")]
    pub epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub next_epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="4")]
    pub prev_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="5")]
    pub outcome_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="6")]
    pub timestamp: u64,
    #[prost(uint64, tag="7")]
    pub timestamp_nanosec: u64,
    #[prost(bytes="vec", tag="8")]
    pub next_bp_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="9")]
    pub block_merkle_root: ::prost::alloc::vec::Vec<u8>,
}
/// Wrapper type over a signature to be able to presenent Option<Signature> inside a Vector
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaybeSignature {
    /// Encoded signature of scheme `Ed25519`
    #[prost(bytes="vec", optional, tag="1")]
    pub signature: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
/// Represents a validator stake state that helps verifying whether a vote is valid or not
/// and if consensus is reached.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStakeView {
    #[prost(uint32, tag="1")]
    pub version: u32,
    #[prost(string, tag="2")]
    pub account_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="3")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    /// NOTE: balance is a u128
    #[prost(bytes="vec", tag="4")]
    pub balance: ::prost::alloc::vec::Vec<u8>,
}
/// ConsensusState defines the consensus state from Tendermint.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(message, optional, tag="1")]
    pub timestamp: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    /// packet commitment root
    #[prost(bytes="vec", tag="2")]
    pub root: ::prost::alloc::vec::Vec<u8>,
}
/// Misbehaviour is a wrapper over two conflicting Headers
/// that implements Misbehaviour interface expected by ICS-02
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehaviour {
    #[prost(message, optional, tag="2")]
    pub header_1: ::core::option::Option<Header>,
    #[prost(message, optional, tag="3")]
    pub header_2: ::core::option::Option<Header>,
}
/// Header contains the neccessary data to proove finality about IBC commitments
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    #[prost(message, optional, tag="1")]
    pub inner: ::core::option::Option<LightClientBlockView>,
}
