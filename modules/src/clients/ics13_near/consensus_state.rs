use serde::Serialize;
use tendermint::Time;
use tendermint_proto::google::protobuf as tpb;
use tendermint_proto::Protobuf;

use super::error::Error;
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;

use ibc_proto::ibc::lightclients::near::v1::ConsensusState as RawConsensusState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NearConsensusState {
    pub commitment_root: CommitmentRoot,
    pub timestamp: Time,
}

impl ConsensusState for NearConsensusState {
    type Error = Error;

    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn root(&self) -> &CommitmentRoot {
        &self.commitment_root
    }

    fn wrap_any(self) -> AnyConsensusState {
        todo!()
    }
}

impl NearConsensusState {
    pub fn new(commitment_root: CommitmentRoot, timestamp: Time) -> Self {
        Self {
            commitment_root,
            timestamp,
        }
    }
}

impl Protobuf<RawConsensusState> for NearConsensusState {}

impl TryFrom<RawConsensusState> for NearConsensusState {
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
            commitment_root: raw.root.into(),
            timestamp,
        })
    }
}

impl From<NearConsensusState> for RawConsensusState {
    fn from(value: NearConsensusState) -> Self {
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: value.commitment_root.into_vec(),
        }
    }
}
