use crate::clients::ics13_near::error::Error;
use crate::core::ics02_client::{
    client_type::ClientType,
    header::{AnyHeader, Header},
};
use ibc_proto::ibc::lightclients::near::v1::{
    signature, validator_stake_view, LightClientBlockView as RawLightClientBlockView,
    NearHeader as RawNearHeader,
};
use near_lite_client::LightClientBlockView;
use near_primitives_wasm::{
    Balance, BlockHeaderInnerLiteView, CryptoHash, Direction, MerkleHash, MerklePath,
    MerklePathItem, PublicKey, Signature, ValidatorStakeView, ValidatorStakeViewV1,
};
use prost::Message;
use sp_core::ed25519::{Public as Ed25519Public, Signature as Ed25519Signature};
use tendermint_proto::Protobuf;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearHeader {
    pub inner: Vec<LightClientBlockView>,
    pub batch_proof: Vec<MerklePath>,
}

impl Header for NearHeader {
    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn wrap_any(self) -> AnyHeader {
        AnyHeader::Near(self)
    }
}

impl TryFrom<RawNearHeader> for NearHeader {
    type Error = Error;

    fn try_from(raw_header: RawNearHeader) -> Result<Self, Self::Error> {
        let inner = raw_header
            .inner
            .into_iter()
            .map(|header| {
                let prev_block_hash = CryptoHash::try_from(header.prev_block_hash.as_slice())?;
                let next_block_inner_hash =
                    CryptoHash::try_from(header.next_block_inner_hash.as_slice())?;
                let inner_rest_hash = CryptoHash::try_from(header.inner_rest_hash.as_slice())?;

                let inner_lite = {
                    let inner_lite = &header.inner_lite.ok_or(Error::invalid_raw_header())?;
                    let epoch_id = CryptoHash::try_from(inner_lite.epoch_id.as_slice())?;
                    let next_epoch_id = CryptoHash::try_from(inner_lite.next_epoch_id.as_slice())?;
                    let prev_state_root =
                        CryptoHash::try_from(inner_lite.prev_state_root.as_slice())?;
                    let outcome_root = CryptoHash::try_from(inner_lite.outcome_root.as_slice())?;
                    let next_bp_hash = CryptoHash::try_from(inner_lite.next_bp_hash.as_slice())?;
                    let block_merkle_root =
                        CryptoHash::try_from(inner_lite.block_merkle_root.as_slice())?;
                    Ok::<_, Error>(BlockHeaderInnerLiteView {
                        height: inner_lite.height,
                        epoch_id,
                        next_epoch_id,
                        prev_state_root,
                        outcome_root,
                        timestamp: inner_lite.timestamp,
                        timestamp_nanosec: inner_lite.timestamp_nanosec,
                        next_bp_hash,
                        block_merkle_root,
                    })
                }?;
                let next_bps = header
                    .next_bps
                    .map(|bps| {
                        Ok::<_, Error>(
                            bps.bps
                                .into_iter()
                                .map(|bp| {
                                    let bp = bp.inner.ok_or(Error::invalid_raw_header())?;
                                    let res = match bp {
                                        validator_stake_view::Inner::V1(stake) => {
                                            let public_key =
                                                PublicKey::try_from(stake.public_key.as_slice())?;
                                            ValidatorStakeView::V1(ValidatorStakeViewV1 {
                                                account_id: stake.account_id,
                                                public_key,
                                                stake: Balance::from_le_bytes(
                                                    stake.stake.try_into().map_err(
                                                        |_e: Vec<u8>| {
                                                            Error::conversion_error(format!(
                                                                "failed to convert bytes to u128"
                                                            ))
                                                        },
                                                    )?,
                                                ),
                                            })
                                        }
                                    };
                                    Ok(res)
                                })
                                .collect::<Result<Vec<_>, Error>>()?,
                        )
                    })
                    .transpose()?;
                let approvals_after_next = header
                    .approvals_after_next
                    .into_iter()
                    .map(|maybe_sig| {
                        maybe_sig
                            .inner
                            .map(|sig| {
                                let sig = sig.inner.ok_or(Error::invalid_raw_header())?;
                                Ok(match sig {
                                    signature::Inner::Ed25519(sig) => Signature::Ed25519(
                                        Ed25519Signature::from_slice(&sig)
                                            .ok_or(Error::invalid_raw_header())?,
                                    ),
                                })
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(LightClientBlockView {
                    prev_block_hash,
                    next_block_inner_hash,
                    inner_lite,
                    inner_rest_hash,
                    next_bps,
                    approvals_after_next,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let batch_proof = raw_header
            .batch_proof
            .into_iter()
            .map(|path| {
                Ok(path
                    .inner
                    .into_iter()
                    .map(|item| {
                        Ok(MerklePathItem {
                            hash: MerkleHash::try_from(item.hash.as_slice())?,
                            direction: if item.direction == 0 {
                                Direction::Left
                            } else {
                                Direction::Right
                            },
                        })
                    })
                    .collect::<Result<_, Error>>()?)
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Self { inner, batch_proof })
    }
}

impl From<NearHeader> for RawNearHeader {
    fn from(beefy_header: NearHeader) -> Self {
        todo!()
    }
}

impl Protobuf<RawNearHeader> for NearHeader {}

pub fn decode_header(buf: &[u8]) -> Result<NearHeader, Error> {
    RawNearHeader::decode(buf)
        .map_err(Error::decode)?
        .try_into()
}
