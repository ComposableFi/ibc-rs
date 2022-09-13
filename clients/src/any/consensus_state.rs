use crate::ics07_tendermint::consensus_state;
use core::convert::Infallible;
use core::fmt::Debug;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::error::Error;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics23_commitment::commitment::CommitmentRoot;
use ibc::prelude::*;
use ibc::timestamp::Timestamp;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;
use serde::Serialize;
use tendermint_proto::Protobuf;

#[cfg(any(test, feature = "ics11_beefy"))]
use crate::ics11_beefy::consensus_state as beefy_consensus_state;

#[cfg(any(test, feature = "ics11_beefy"))]
use crate::ics13_near::consensus_state as near_consensus_state;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::MockConsensusState;

pub const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";
pub const BEEFY_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ConsensusState";
pub const NEAR_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.near.v1.ConsensusState";
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
    #[ibc(proto_url = "TENDERMINT_CONSENSUS_STATE_TYPE_URL")]
    Tendermint(consensus_state::ConsensusState),
    #[cfg(any(test, feature = "ics11_beefy"))]
    #[ibc(proto_url = "BEEFY_CONSENSUS_STATE_TYPE_URL")]
    Beefy(beefy_consensus_state::ConsensusState),
    #[cfg(any(test, feature = "ics11_beefy"))]
    #[ibc(proto_url = "NEAR_CONSENSUS_STATE_TYPE_URL")]
    Near(near_consensus_state::ConsensusState),
    #[cfg(any(test, feature = "mocks"))]
    #[ibc(proto_url = "MOCK_CONSENSUS_STATE_TYPE_URL")]
    Mock(MockConsensusState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnyConsensusStateWithHeight {
    pub height: Height,
    pub consensus_state: AnyConsensusState,
}

impl Protobuf<ConsensusStateWithHeight> for AnyConsensusStateWithHeight {}

impl TryFrom<ConsensusStateWithHeight> for AnyConsensusStateWithHeight {
    type Error = Error;

    fn try_from(value: ConsensusStateWithHeight) -> Result<Self, Self::Error> {
        let state = value
            .consensus_state
            .map(AnyConsensusState::try_from)
            .transpose()?
            .ok_or_else(Error::empty_consensus_state_response)?;

        Ok(AnyConsensusStateWithHeight {
            height: value.height.ok_or_else(Error::missing_height)?.into(),
            consensus_state: state,
        })
    }
}

impl From<AnyConsensusStateWithHeight> for ConsensusStateWithHeight {
    fn from(value: AnyConsensusStateWithHeight) -> Self {
        ConsensusStateWithHeight {
            height: Some(value.height.into()),
            consensus_state: Some(value.consensus_state.into()),
        }
    }
}
