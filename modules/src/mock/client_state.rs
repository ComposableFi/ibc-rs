use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap as HashMap;

use core::convert::Infallible;
use core::fmt::Debug;
use core::time::Duration;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::ibc::mock::ConsensusState as RawMockConsensusState;

use crate::clients::{ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_def::AnyGlobalDef;
use crate::core::ics02_client::client_state::{AnyClientState, ClientState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::ics24_host::identifier::ChainId;
use crate::mock::client_def::{MockClient, TestGlobalDefs};
use crate::mock::header::MockHeader;
use crate::test_utils::Crypto;
use crate::timestamp::Timestamp;
use crate::{downcast, Height};
use derivative::Derivative;

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The type of this client.
    pub client_type: ClientType,

    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<AnyClientState<TestGlobalDefs>>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: HashMap<Height, AnyConsensusState>,
}

/// A mock of a client state. For an example of a real structure that this mocks, you can see
/// `ClientState` of ics07_tendermint/client_state.rs.
#[derive(Serialize, Deserialize, Derivative)]
#[derivative(
    Copy(bound = ""),
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct MockClientState<G> {
    pub header: MockHeader,
    pub frozen_height: Option<Height>,
    pub _phantom: PhantomData<G>,
}

impl<G> Protobuf<RawMockClientState> for MockClientState<G> {}

impl<G> MockClientState<G> {
    pub fn new(header: MockHeader) -> Self {
        Self {
            header,
            frozen_height: None,
            _phantom: PhantomData,
        }
    }

    pub fn refresh_time(&self) -> Option<Duration> {
        None
    }
}

impl<G> From<MockClientState<G>> for AnyClientState<G> {
    fn from(mcs: MockClientState<G>) -> Self {
        Self::Mock(mcs)
    }
}

impl<G> TryFrom<RawMockClientState> for MockClientState<G> {
    type Error = Error;

    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.header.unwrap().try_into()?))
    }
}

impl<G> From<MockClientState<G>> for RawMockClientState {
    fn from(value: MockClientState<G>) -> Self {
        RawMockClientState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.nanoseconds(),
            }),
        }
    }
}

impl<G: GlobalDefs + Clone> ClientState for MockClientState<G>
where
    MockConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<MockConsensusState>,
{
    type UpgradeOptions = ();
    type ClientDef = MockClient<G>;

    fn chain_id(&self) -> ChainId {
        self.chain_id()
    }

    fn client_type(&self) -> ClientType {
        self.client_type()
    }

    fn client_def(&self) -> Self::ClientDef {
        self.client_def()
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
}

impl<G> MockClientState<G> {
    pub fn chain_id(&self) -> ChainId {
        ChainId::default()
    }

    pub fn client_type(&self) -> ClientType {
        ClientType::Mock
    }

    pub fn client_def(&self) -> MockClient<G> {
        todo!()
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

    pub fn expired(&self, elapsed: Duration) -> bool {
        false
    }
}

impl<G> From<MockConsensusState> for MockClientState<G> {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
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

    fn wrap_any(self) -> AnyConsensusState {
        AnyConsensusState::Mock(self)
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp()
    }
}
