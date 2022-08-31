use super::error::Error;
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::timestamp::Timestamp;
use near_lite_client::{CryptoHash, LightClientBlockView};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct NearConsensusState {
    commitment_root: CommitmentRoot,
    pub(crate) timestamp: Timestamp,
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
                .map_err(|_| Error::invalid_timestamp())?,
        })
    }
}
