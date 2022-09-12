use crate::prelude::*;

use core::convert::Infallible;
use core::fmt::Debug;
use core::marker::{Send, Sync};

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;
use serde::Serialize;
use tendermint_proto::Protobuf;

use crate::clients::ics07_tendermint::consensus_state;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::consensus_state as beefy_consensus_state;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::height::Height;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::WithBlockDataType;
use crate::timestamp::Timestamp;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::MockConsensusState;

pub const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";

pub const BEEFY_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ConsensusState";

pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

pub trait ConsensusState: Clone + Debug + Send + Sync {
    type Error;

    /// Type of client associated with this consensus state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Commitment root of the consensus state, which is used for key-value pair verification.
    fn root(&self) -> &CommitmentRoot;

    /// Returns the timestamp of the state.
    fn timestamp(&self) -> Timestamp;

    fn downcast<T: Clone + 'static>(self) -> T
    where
        Self: 'static,
    {
        <dyn core::any::Any>::downcast_ref(&self)
            .cloned()
            .expect("downcast failed")
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Self
    where
        Self: 'static,
    {
        sub_state
            .downcast_ref::<Self>()
            .expect("ConsensusState wrap failed")
            .clone()
    }

    fn encode_to_vec(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
    Tendermint(consensus_state::ConsensusState),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Beefy(beefy_consensus_state::ConsensusState),
    #[cfg(any(test, feature = "mocks"))]
    Mock(MockConsensusState),
}

impl AnyConsensusState {
    pub fn client_type(&self) -> ClientType {
        match self {
            AnyConsensusState::Tendermint(_cs) => ClientType::Tendermint,
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyConsensusState::Beefy(_) => ClientType::Beefy,
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(_cs) => ClientType::Mock,
        }
    }
}

impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "" => Err(Error::empty_consensus_state_response()),

            TENDERMINT_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Tendermint(
                consensus_state::ConsensusState::decode_vec(&value.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            #[cfg(any(test, feature = "ics11_beefy"))]
            BEEFY_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Beefy(
                beefy_consensus_state::ConsensusState::decode_vec(&value.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            #[cfg(any(test, feature = "mocks"))]
            MOCK_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Mock(
                MockConsensusState::decode_vec(&value.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            _ => Err(Error::unknown_consensus_state_type(value.type_url)),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        match value {
            AnyConsensusState::Tendermint(value) => Any {
                type_url: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: value.encode_to_vec(),
            },
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyConsensusState::Beefy(value) => Any {
                type_url: BEEFY_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: value.encode_to_vec(),
            },
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(value) => Any {
                type_url: MOCK_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: value.encode_to_vec(),
            },
        }
    }
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

impl ConsensusState for AnyConsensusState {
    type Error = Infallible;

    fn client_type(&self) -> ClientType {
        self.client_type()
    }

    fn root(&self) -> &CommitmentRoot {
        match self {
            Self::Tendermint(cs_state) => cs_state.root(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(cs_state) => cs_state.root(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.root(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::Tendermint(cs_state) => cs_state.timestamp(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(cs_state) => cs_state.timestamp(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.timestamp(),
        }
    }

    fn downcast<T: Clone + 'static>(self) -> T {
        match self {
            Self::Tendermint(cs_state) => cs_state.downcast(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(cs_state) => cs_state.downcast(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.downcast(),
        }
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Self {
        if let Some(state) = sub_state.downcast_ref::<consensus_state::ConsensusState>() {
            return AnyConsensusState::Tendermint(state.clone());
        }
        #[cfg(any(test, feature = "ics11_beefy"))]
        if let Some(state) = sub_state.downcast_ref::<beefy_consensus_state::ConsensusState>() {
            return AnyConsensusState::Beefy(state.clone());
        }
        // TODO: wrap near consensus state
        #[cfg(any(test, feature = "mocks"))]
        if let Some(state) = sub_state.downcast_ref::<MockConsensusState>() {
            return AnyConsensusState::Mock(state.clone());
        }
        panic!("unknown consensus state type")
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        self.encode_vec()
    }
}

/// Query request for a single client event, identified by `event_id`, for `client_id`.
#[derive(Clone, Debug)]
pub struct QueryClientEventRequest {
    pub height: crate::Height,
    pub event_id: WithBlockDataType,
    pub client_id: ClientId,
    pub consensus_height: crate::Height,
}
