use crate::prelude::*;

use core::convert::Infallible;
use serde::Serialize;
use sp_runtime::SaturatedConversion;
use tendermint::time::Time;
use tendermint_proto::google::protobuf as tpb;
use tendermint_proto::Protobuf;

use ibc_proto::ibc::lightclients::beefy::v1::ConsensusState as RawConsensusState;

use crate::clients::ics11_beefy::error::Error;
use crate::clients::ics11_beefy::header::{decode_timestamp_extrinsic, ParachainHeader};
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::timestamp::Timestamp;

// This is a constant that comes from pallet-ibc
pub const IBC_CONSENSUS_ID: [u8; 4] = *b"/IBC";
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: CommitmentRoot,
}

impl ConsensusState {
    pub fn new(root: Vec<u8>, timestamp: Time) -> Self {
        Self {
            timestamp,
            root: root.into(),
        }
    }
}

impl crate::core::ics02_client::client_consensus::ConsensusState for ConsensusState {
    type Error = Infallible;

    fn client_type(&self) -> ClientType {
        ClientType::Beefy
    }

    fn root(&self) -> &CommitmentRoot {
        &self.root
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
            root: raw.root.into(),
            timestamp,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: value.root.into_vec(),
        }
    }
}

impl TryFrom<ParachainHeader> for ConsensusState {
    type Error = Error;
    fn try_from(header: ParachainHeader) -> Result<Self, Self::Error> {
        let root = {
            header
                .parachain_header
                .digest
                .logs
                .iter()
                .filter_map(|digest| digest.as_consensus())
                .find(|(id, _value)| id == &IBC_CONSENSUS_ID)
                .map(|(.., root)| root.to_vec())
                .ok_or(Error::invalid_header("cannot find ibc commitment root".to_string()))?
        };

        let timestamp = decode_timestamp_extrinsic(&header).unwrap_or_default();
        let duration = core::time::Duration::from_millis(timestamp);
        let timestamp = Timestamp::from_nanoseconds(duration.as_nanos().saturated_into::<u64>())
            .unwrap_or_default()
            .into_tm_time()
            .ok_or(Error::invalid_header("cannot decode timestamp extrinsic".to_string()))?;

        Ok(Self {
            root: root.into(),
            timestamp,
        })
    }
}

#[cfg(any(test, feature = "mocks"))]
pub mod test_util {
    use super::*;

    pub fn get_dummy_beefy_consensus_state() -> AnyConsensusState {
        AnyConsensusState::Beefy(ConsensusState {
            timestamp: Time::now(),
            root: vec![0; 32].into(),
        })
    }
}
#[cfg(test)]
mod tests {}
