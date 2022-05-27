use serde::Serialize;
use tendermint::Time;
use tendermint_proto::Protobuf;

use super::error::Error;
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;

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

// TODO: we need to define the RawConsensusState for NearConsensusState.
// david mentioned that we will need to get help from seun
impl Protobuf<RawConsensusState> for NearConsensusState {}
