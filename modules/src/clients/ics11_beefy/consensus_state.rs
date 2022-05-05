use crate::prelude::*;

use beefy_client::primitives::PartialMmrLeaf;
use beefy_primitives::mmr::{BeefyNextAuthoritySet, MmrLeafVersion};
use codec::Encode;
use core::convert::Infallible;

use serde::Serialize;
use sp_core::H256;
use sp_runtime::SaturatedConversion;
use tendermint::{hash::Algorithm, time::Time, Hash};
use tendermint_proto::google::protobuf as tpb;
use tendermint_proto::Protobuf;

use ibc_proto::ibc::lightclients::beefy::v1::{
    BeefyAuthoritySet, BeefyMmrLeafPartial as RawPartialMmrLeaf,
    ConsensusState as RawConsensusState, ParachainHeader as RawParachainHeader,
};

use crate::clients::ics11_beefy::error::Error;
use crate::clients::ics11_beefy::header::{
    decode_parachain_header, decode_timestamp_extrinsic, merge_leaf_version, split_leaf_version,
    ParachainHeader,
};
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::timestamp::Timestamp;

pub const IBC_CONSENSUS_ID: [u8; 4] = *b"/IBC";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: Vec<u8>
}

impl ConsensusState {
    pub fn new(root: Vec<u8>, timestamp: Time) -> Self {
        Self {
            timestamp,
            root
        }
    }
}

impl crate::core::ics02_client::client_consensus::ConsensusState for ConsensusState {
    type Error = Infallible;

    fn client_type(&self) -> ClientType {
        ClientType::Beefy
    }

    fn root(&self) -> &CommitmentRoot {
        &self.root.into()
    }

    fn wrap_any(self) -> AnyConsensusState {
        AnyConsensusState::Beefy(self)
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        let ibc_proto::google::protobuf::Timestamp { seconds, nanos } = raw
            .timestamp
            .ok_or_else(|| Error::invalid_raw_consensus_state("missing timestamp".into()))?;
        let proto_timestamp = tpb::Timestamp { seconds, nanos };
        let timestamp = proto_timestamp
            .try_into()
            .map_err(|e| Error::invalid_raw_consensus_state(format!("invalid timestamp: {}", e)))?;

        Ok(Self {
            root: raw.root,
            timestamp
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: value.root
        }
    }
}

impl From<ParachainHeader> for ConsensusState {
    fn from(header: ParachainHeader) -> Self {
        let root = {
            header
                .parachain_header
                .digest
                .logs
                .into_iter()
                .filter_map(|digest| digest.as_consensus())
                .find(|(id, value)| id == &IBC_CONSENSUS_ID)
                .map(|(.., root)| root.to_vec())
                .unwrap_or_default()
        };

        let timestamp = decode_timestamp_extrinsic(&header).unwrap_or_default();
        let duration = core::time::Duration::from_millis(timestamp);
        let timestamp = Timestamp::from_nanoseconds(duration.as_nanos().saturated_into::<u64>())
            .unwrap_or_default();

        Self {
            root,
            timestamp: timestamp.into()
        }
    }
}

#[cfg(test)]
mod tests {}
