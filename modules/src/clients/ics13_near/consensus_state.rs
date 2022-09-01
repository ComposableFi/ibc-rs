use super::error::Error;
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::timestamp::Timestamp;
use ibc_proto::ibc::lightclients::near::v1::ConsensusState as RawConsensusState;
use near_lite_client::{CryptoHash, LightClientBlockView};
use serde::Serialize;
use tendermint::Time;
use tendermint_proto::Protobuf;

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct NearConsensusState {
    pub(crate) timestamp: Time,
    commitment_root: CommitmentRoot,
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
    /// To construct a consensus state form a header, it's also
    /// required to provide a state root hash, which is only available
    /// for the previous block, so it should be passed separately.
    pub fn from_header(
        lite_block: &LightClientBlockView,
        state_root: CryptoHash,
    ) -> Result<Self, Error> {
        Ok(Self {
            commitment_root: CommitmentRoot::from_bytes(state_root.as_bytes()),
            timestamp: Timestamp::from_nanoseconds(lite_block.inner_lite.timestamp)
                .map_err(|_| Error::invalid_timestamp())?
                .into_tm_time()
                .ok_or_else(|| Error::invalid_timestamp())?,
        })
    }
}

impl Protobuf<RawConsensusState> for NearConsensusState {}

impl TryFrom<RawConsensusState> for NearConsensusState {
    type Error = Error;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        let ibc_proto::google::protobuf::Timestamp { seconds, nanos } = raw
            .timestamp
            .ok_or_else(|| Error::invalid_raw_consensus_state("missing timestamp".into()))?;
        let proto_timestamp = tendermint_proto::google::protobuf::Timestamp { seconds, nanos };
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
        let tendermint_proto::google::protobuf::Timestamp { seconds, nanos } =
            value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: value.commitment_root.into_vec(),
        }
    }
}
