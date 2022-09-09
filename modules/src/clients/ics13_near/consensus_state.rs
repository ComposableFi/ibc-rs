use super::error::Error;
use crate::core::ics02_client::client_consensus::{self, AnyConsensusState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::timestamp::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusState {
    commitment_root: CommitmentRoot,
}

impl client_consensus::ConsensusState for ConsensusState {
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

    fn timestamp(&self) -> Timestamp {
        todo!()
    }

    fn downcast<T: Clone + 'static>(self) -> T {
        todo!()
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Self {
        todo!()
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        todo!()
    }
}
