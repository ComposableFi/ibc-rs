use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap as HashMap;

use core::convert::Infallible;
use core::fmt::Debug;
use core::time::Duration;

use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;
use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::ibc::mock::ConsensusState as RawMockConsensusState;

use crate::core::ics02_client::client_consensus::ConsensusState;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::ics24_host::identifier::ChainId;
use crate::mock::context::ClientTypes;
use crate::mock::header::MockHeader;
use crate::timestamp::Timestamp;
use crate::{downcast, Height};
use ibc_proto::google::protobuf::Any;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord<C: ClientTypes> {
    /// The type of this client.
    pub client_type: ClientType,

    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<C::AnyClientState>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: HashMap<Height, C::AnyConsensusState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnyUpgradeOptions {
    Mock(()),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ClientState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyClientState {
    #[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
    Mock(MockClientState),
}

/// A mock of a client state. For an example of a real structure that this mocks, you can see
/// `ClientState` of ics07_tendermint/client_state.rs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub struct MockClientState {
    pub header: MockHeader,
    pub frozen_height: Option<Height>,
}

impl Protobuf<RawMockClientState> for MockClientState {}

impl MockClientState {
    pub fn new(header: MockHeader) -> Self {
        Self {
            header,
            frozen_height: None,
        }
    }

    pub fn refresh_time(&self) -> Option<Duration> {
        None
    }
}

impl From<MockClientState> for AnyClientState {
    fn from(mcs: MockClientState) -> Self {
        Self::Mock(mcs)
    }
}

impl TryFrom<RawMockClientState> for MockClientState {
    type Error = Error;

    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.header.unwrap().try_into()?))
    }
}

impl From<MockClientState> for RawMockClientState {
    fn from(value: MockClientState) -> Self {
        RawMockClientState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.nanoseconds(),
            }),
        }
    }
}

impl ClientState for MockClientState {
    type UpgradeOptions = ();

    fn chain_id(&self) -> ChainId {
        self.chain_id()
    }

    fn client_type(&self) -> ClientType {
        self.client_type()
    }

    fn latest_height(&self) -> Height {
        self.latest_height()
    }

    fn frozen_height(&self) -> Option<Height> {
        self.frozen_height()
    }

    fn upgrade(self, _upgrade_height: Height, _upgrade_options: (), _chain_id: ChainId) -> Self {
        self.upgrade(_upgrade_height, _upgrade_options, _chain_id)
    }

    fn expired(&self, elapsed: Duration) -> bool {
        self.expired(elapsed)
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        self.encode_vec()
    }
}

impl MockClientState {
    pub fn chain_id(&self) -> ChainId {
        ChainId::default()
    }

    pub fn client_type(&self) -> ClientType {
        ClientType::Mock
    }

    pub fn latest_height(&self) -> Height {
        self.header.height()
    }

    pub fn frozen_height(&self) -> Option<Height> {
        self.frozen_height
    }

    pub fn upgrade(
        self,
        _upgrade_height: Height,
        _upgrade_options: (),
        _chain_id: ChainId,
    ) -> Self {
        todo!()
    }

    pub fn expired(&self, _elapsed: Duration) -> bool {
        false
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}

pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ConsensusState, Protobuf)]
#[serde(tag = "type")]
pub enum AnyConsensusState {
    #[ibc(proto_url = "MOCK_CONSENSUS_STATE_TYPE_URL")]
    Mock(MockConsensusState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnyConsensusStateWithHeight<C: ClientTypes> {
    pub height: Height,
    pub consensus_state: C::AnyConsensusState,
}

impl<C: ClientTypes> Protobuf<ConsensusStateWithHeight> for AnyConsensusStateWithHeight<C> {}

impl<C: ClientTypes> TryFrom<ConsensusStateWithHeight> for AnyConsensusStateWithHeight<C> {
    type Error = Error;

    fn try_from(value: ConsensusStateWithHeight) -> Result<Self, Self::Error> {
        let state = value
            .consensus_state
            .map(C::AnyConsensusState::try_from)
            .transpose()?
            .ok_or_else(Error::empty_consensus_state_response)?;

        Ok(AnyConsensusStateWithHeight {
            height: value.height.ok_or_else(Error::missing_height)?.into(),
            consensus_state: state,
        })
    }
}

impl<C: ClientTypes> From<AnyConsensusStateWithHeight<C>> for ConsensusStateWithHeight {
    fn from(value: AnyConsensusStateWithHeight<C>) -> Self {
        ConsensusStateWithHeight {
            height: Some(value.height.into()),
            consensus_state: Some(value.consensus_state.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MockConsensusState {
    pub header: MockHeader,
    pub root: CommitmentRoot,
}

impl MockConsensusState {
    pub fn new(header: MockHeader) -> Self {
        MockConsensusState {
            header,
            root: CommitmentRoot::from(vec![0]),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.header.timestamp
    }
}

impl Protobuf<RawMockConsensusState> for MockConsensusState {}

impl TryFrom<RawMockConsensusState> for MockConsensusState {
    type Error = Error;

    fn try_from(raw: RawMockConsensusState) -> Result<Self, Self::Error> {
        let raw_header = raw.header.ok_or_else(Error::missing_raw_consensus_state)?;

        Ok(Self {
            header: MockHeader::try_from(raw_header)?,
            root: CommitmentRoot::from(vec![0]),
        })
    }
}

impl From<MockConsensusState> for RawMockConsensusState {
    fn from(value: MockConsensusState) -> Self {
        RawMockConsensusState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.nanoseconds(),
            }),
        }
    }
}

impl From<MockConsensusState> for AnyConsensusState {
    fn from(mcs: MockConsensusState) -> Self {
        Self::Mock(mcs)
    }
}

impl TryFrom<AnyConsensusState> for MockConsensusState {
    type Error = Error;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        downcast!(
            value => AnyConsensusState::Mock
        )
        .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))
    }
}

impl ConsensusState for MockConsensusState {
    type Error = Infallible;

    fn client_type(&self) -> ClientType {
        ClientType::Mock
    }

    fn root(&self) -> &CommitmentRoot {
        &self.root
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp()
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        self.encode_vec()
    }
}
