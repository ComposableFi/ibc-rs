#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePathItem {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration = "Direction", tag = "2")]
    pub direction: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Signature {
    #[prost(oneof = "signature::Inner", tags = "1")]
    pub inner: ::core::option::Option<signature::Inner>,
}
/// Nested message and enum types in `Signature`.
pub mod signature {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Inner {
        #[prost(bytes, tag = "1")]
        Ed25519(::prost::alloc::vec::Vec<u8>),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStakeView {
    #[prost(oneof = "validator_stake_view::Inner", tags = "1")]
    pub inner: ::core::option::Option<validator_stake_view::Inner>,
}
/// Nested message and enum types in `ValidatorStakeView`.
pub mod validator_stake_view {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Inner {
        #[prost(message, tag = "1")]
        V1(super::ValidatorStakeViewV1),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaybeSignature {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<Signature>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockProducers {
    #[prost(message, repeated, tag = "1")]
    pub bps: ::prost::alloc::vec::Vec<ValidatorStakeView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientBlockView {
    #[prost(bytes = "vec", tag = "1")]
    pub prev_block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub next_block_inner_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub inner_lite: ::core::option::Option<BlockHeaderInnerLiteView>,
    #[prost(bytes = "vec", tag = "4")]
    pub inner_rest_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "5")]
    pub next_bps: ::core::option::Option<BlockProducers>,
    #[prost(message, repeated, tag = "6")]
    pub approvals_after_next: ::prost::alloc::vec::Vec<MaybeSignature>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeaderInnerLiteView {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub next_epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    pub prev_state_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "5")]
    pub outcome_root: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "6")]
    pub timestamp: u64,
    #[prost(uint64, tag = "7")]
    pub timestamp_nanosec: u64,
    #[prost(bytes = "vec", tag = "8")]
    pub next_bp_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "9")]
    pub block_merkle_root: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStakeViewV1 {
    #[prost(string, tag = "1")]
    pub account_id: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    /// uint128
    #[prost(bytes = "vec", tag = "3")]
    pub stake: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MerklePath {
    #[prost(message, repeated, tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<MerklePathItem>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NearHeader {
    #[prost(message, repeated, tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<LightClientBlockView>,
    #[prost(message, repeated, tag = "2")]
    pub batch_proof: ::prost::alloc::vec::Vec<MerklePath>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NearBlockProducersForEpoch {
    #[prost(bytes = "vec", tag = "1")]
    pub epoch_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag = "2")]
    pub validator_stakes: ::prost::alloc::vec::Vec<ValidatorStakeView>,
}
/// ClientState from Beefy tracks the current validator set, latest height,
/// and a possible frozen height.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    /// Latest known block
    #[prost(message, optional, tag = "1")]
    pub head: ::core::option::Option<LightClientBlockView>,
    /// Current epoch number
    #[prost(uint64, tag = "2")]
    pub epoch_num: u64,
    /// Previous epoch number
    #[prost(uint64, tag = "3")]
    pub prev_epoch_num: u64,
    /// Current epoch
    #[prost(bytes = "vec", tag = "4")]
    pub current_epoch: ::prost::alloc::vec::Vec<u8>,
    /// Next epoch
    #[prost(bytes = "vec", tag = "5")]
    pub next_epoch: ::prost::alloc::vec::Vec<u8>,
    //// Block producers for an epoch
    #[prost(message, repeated, tag = "6")]
    pub epoch_block_producers: ::prost::alloc::vec::Vec<NearBlockProducersForEpoch>,
    //// Previous (to the head) lite block
    #[prost(message, repeated, tag = "7")]
    pub prev_lite_block: ::prost::alloc::vec::Vec<LightClientBlockView>,
    /// Block height when the client was frozen due to a misbehaviour
    #[prost(uint64, tag = "8")]
    pub frozen_height: u64,
}
/// ConsensusState defines the consensus state from Tendermint.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    /// packet commitment root
    #[prost(bytes = "vec", tag = "2")]
    pub root: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Direction {
    Left = 0,
    Right = 1,
}
