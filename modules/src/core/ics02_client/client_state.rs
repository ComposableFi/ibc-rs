use core::fmt::{Debug, Display};
use core::marker::{Send, Sync};
use core::time::Duration;

use derivative::Derivative;
use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use crate::clients::ics07_tendermint::client_def::TendermintClient;
use crate::clients::ics07_tendermint::client_state;
use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TendermintConsensusState;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::client_state as beefy_client_state;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::{
    client_def::BeefyClient, consensus_state::ConsensusState as BeefyConsensusState,
};
use crate::clients::{ClientStateOf, ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_def::{AnyClient, ClientDef};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::trust_threshold::TrustThreshold;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_def::MockClient;
#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::{MockClientState, MockConsensusState};
use crate::prelude::*;
use crate::Height;
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;

#[cfg(not(feature = "ics11_beefy"))]
use super::client_def::stub_beefy::*;
#[cfg(not(test))]
use super::client_def::stub_mock::*;

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";
pub const BEEFY_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ClientState";
pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

pub trait ClientState: Clone + Debug + Send + Sync {
    /// Client-specific options for upgrading the client
    type UpgradeOptions;

    /// Client definition type (used for verification)
    type ClientDef: ClientDef;

    /// Return the chain identifier which this client is serving (i.e., the client is verifying
    /// consensus states from this chain).
    fn chain_id(&self) -> ChainId;

    /// Type of client associated with this state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Returns a client definition for this client state
    fn client_def(&self) -> Self::ClientDef;

    /// Latest height of consensus state
    fn latest_height(&self) -> Height;

    /// Freeze status of the client
    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    /// Frozen height of the client
    fn frozen_height(&self) -> Option<Height>;

    /// Helper function to verify the upgrade client procedure.
    /// Resets all fields except the blockchain-specific ones,
    /// and updates the given fields.
    fn upgrade(
        self,
        upgrade_height: Height,
        upgrade_options: Self::UpgradeOptions,
        chain_id: ChainId,
    ) -> Self;

    /// Helper function to verify the upgrade client procedure.
    fn expired(&self, elapsed: Duration) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnyUpgradeOptions {
    Tendermint(client_state::UpgradeOptions),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Beefy(beefy_client_state::UpgradeOptions),
    #[cfg(any(test, feature = "mocks"))]
    Mock(()),
}

impl AnyUpgradeOptions {
    fn into_tendermint(self) -> client_state::UpgradeOptions {
        match self {
            Self::Tendermint(options) => options,
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(_) => {
                panic!("cannot downcast AnyUpgradeOptions::Beefy to Tendermint::UpgradeOptions")
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(_) => {
                panic!("cannot downcast AnyUpgradeOptions::Mock to Tendermint::UpgradeOptions")
            }
        }
    }

    #[cfg(any(test, feature = "ics11_beefy"))]
    fn into_beefy(self) -> beefy_client_state::UpgradeOptions {
        match self {
            Self::Tendermint(_) => {
                panic!("cannot downcast AnyUpgradeOptions::Tendermint to Beefy::UpgradeOptions")
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(options) => options,
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(_) => {
                panic!("cannot downcast AnyUpgradeOptions::Mock to Tendermint::UpgradeOptions")
            }
        }
    }
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
#[serde(tag = "type")]
pub enum AnyClientState<G> {
    Tendermint(client_state::ClientState<G>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    #[serde(skip)]
    Beefy(beefy_client_state::ClientState<G>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    #[serde(skip)]
    Near(beefy_client_state::ClientState<G>),
    #[cfg(any(test, feature = "mocks"))]
    Mock(MockClientState<G>),
}

impl<G> AnyClientState<G> {
    pub fn latest_height(&self) -> Height {
        match self {
            Self::Tendermint(tm_state) => tm_state.latest_height(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(bf_state) => bf_state.latest_height(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.latest_height(),
        }
    }

    pub fn frozen_height(&self) -> Option<Height> {
        match self {
            Self::Tendermint(tm_state) => tm_state.frozen_height(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(bf_state) => bf_state.frozen_height(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.frozen_height(),
        }
    }

    pub fn trust_threshold(&self) -> Option<TrustThreshold> {
        match self {
            AnyClientState::Tendermint(state) => Some(state.trust_level),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(_) => None,
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(_) => None,
        }
    }

    pub fn max_clock_drift(&self) -> Duration {
        match self {
            AnyClientState::Tendermint(state) => state.max_clock_drift,
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(_) => Duration::new(0, 0),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(_) => Duration::new(0, 0),
        }
    }

    pub fn client_type(&self) -> ClientType {
        match self {
            Self::Tendermint(state) => state.client_type(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(state) => state.client_type(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(state) => state.client_type(),
        }
    }

    pub fn refresh_period(&self) -> Option<Duration> {
        match self {
            AnyClientState::Tendermint(tm_state) => tm_state.refresh_time(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(_) => None,
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => None,
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => mock_state.refresh_time(),
        }
    }

    pub fn expired(&self, elapsed_since_latest: Duration) -> bool {
        match self {
            AnyClientState::Tendermint(tm_state) => tm_state.expired(elapsed_since_latest),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(bf_state) => bf_state.expired(elapsed_since_latest),
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => false,
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => mock_state.expired(elapsed_since_latest),
        }
    }
}

impl<G> Protobuf<Any> for AnyClientState<G> {}

impl<G> TryFrom<Any> for AnyClientState<G> {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            "" => Err(Error::empty_client_state_response()),

            TENDERMINT_CLIENT_STATE_TYPE_URL => Ok(AnyClientState::Tendermint(
                client_state::ClientState::decode_vec(&raw.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            #[cfg(any(test, feature = "ics11_beefy"))]
            BEEFY_CLIENT_STATE_TYPE_URL => Ok(AnyClientState::Beefy(
                beefy_client_state::ClientState::decode_vec(&raw.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            #[cfg(any(test, feature = "mocks"))]
            MOCK_CLIENT_STATE_TYPE_URL => Ok(AnyClientState::Mock(
                MockClientState::decode_vec(&raw.value).map_err(Error::decode_raw_client_state)?,
            )),

            _ => Err(Error::unknown_client_state_type(raw.type_url)),
        }
    }
}

impl<G> From<AnyClientState<G>> for Any {
    fn from(value: AnyClientState<G>) -> Self {
        match value {
            AnyClientState::Tendermint(value) => Any {
                type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                value: value.encode_vec(),
            },
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(value) => Any {
                type_url: BEEFY_CLIENT_STATE_TYPE_URL.to_string(),
                value: value.encode_vec(),
            },
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Near(_) => Any {
                type_url: BEEFY_CLIENT_STATE_TYPE_URL.to_string(),
                value: value.encode_vec(),
            },
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(value) => Any {
                type_url: MOCK_CLIENT_STATE_TYPE_URL.to_string(),
                value: value.encode_vec(),
            },
        }
    }
}

impl<G> ClientState for AnyClientState<G>
where
    G: GlobalDefs + Clone,
    G::HostFunctions: Sync + Send + Clone + Debug + Eq,

    TendermintConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<TendermintConsensusState>,

    BeefyConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<BeefyConsensusState>,

    MockConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<MockConsensusState>,

    ConsensusStateOf<G>: Protobuf<Any>,
    ConsensusStateOf<G>: TryFrom<Any>,
    <ConsensusStateOf<G> as TryFrom<Any>>::Error: Display,
    Any: From<ConsensusStateOf<G>>,

    ClientStateOf<G>: Protobuf<Any>,
    ClientStateOf<G>: TryFrom<Any>,
    <ClientStateOf<G> as TryFrom<Any>>::Error: Display,
    Any: From<ClientStateOf<G>>,
{
    type UpgradeOptions = AnyUpgradeOptions;
    type ClientDef = AnyClient<G>;

    fn chain_id(&self) -> ChainId {
        match self {
            AnyClientState::Tendermint(tm_state) => tm_state.chain_id(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(bf_state) => bf_state.chain_id(),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => mock_state.chain_id(),
        }
    }

    fn client_type(&self) -> ClientType {
        self.client_type()
    }

    fn client_def(&self) -> Self::ClientDef {
        match self {
            AnyClientState::Tendermint(_tm_state) => {
                AnyClient::Tendermint(TendermintClient::default())
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(_bf_state) => AnyClient::Beefy(BeefyClient::default()),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => AnyClient::Mock(MockClient::default()),
        }
    }

    fn latest_height(&self) -> Height {
        self.latest_height()
    }

    fn frozen_height(&self) -> Option<Height> {
        self.frozen_height()
    }

    fn upgrade(
        self,
        upgrade_height: Height,
        upgrade_options: Self::UpgradeOptions,
        chain_id: ChainId,
    ) -> Self {
        match self {
            AnyClientState::Tendermint(tm_state) => AnyClientState::Tendermint(tm_state.upgrade(
                upgrade_height,
                upgrade_options.into_tendermint(),
                chain_id,
            )),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(bf_state) => AnyClientState::Beefy(bf_state.upgrade(
                upgrade_height,
                upgrade_options.into_beefy(),
                chain_id,
            )),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Near(near_state) => AnyClientState::Near(near_state.upgrade(
                upgrade_height,
                upgrade_options.into_beefy(),
                chain_id,
            )),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => {
                AnyClientState::Mock(mock_state.upgrade(upgrade_height, (), chain_id))
            }
        }
    }

    fn expired(&self, elapsed: Duration) -> bool {
        match self {
            AnyClientState::Tendermint(tm_state) => tm_state.expired(elapsed),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Beefy(bf_state) => bf_state.expired(elapsed),
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClientState::Near(_) => todo!(),
            #[cfg(any(test, feature = "mocks"))]
            AnyClientState::Mock(mock_state) => mock_state.expired(elapsed),
        }
    }
}

#[derive(Derivative, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[derivative(Clone(bound = ""))]
#[serde(tag = "type")]
pub struct IdentifiedAnyClientState<G> {
    pub client_id: ClientId,
    pub client_state: AnyClientState<G>,
}

impl<G> IdentifiedAnyClientState<G> {
    pub fn new(client_id: ClientId, client_state: AnyClientState<G>) -> Self {
        IdentifiedAnyClientState {
            client_id,
            client_state,
        }
    }
}

impl<G> Protobuf<IdentifiedClientState> for IdentifiedAnyClientState<G>
where
    IdentifiedAnyClientState<G>: TryFrom<IdentifiedClientState>,
    <IdentifiedAnyClientState<G> as TryFrom<IdentifiedClientState>>::Error: Display,
{
}

impl<G> TryFrom<IdentifiedClientState> for IdentifiedAnyClientState<G>
where
    AnyClientState<G>: TryFrom<Any>,
    Error: From<<AnyClientState<G> as TryFrom<Any>>::Error>,
{
    type Error = Error;

    fn try_from(raw: IdentifiedClientState) -> Result<Self, Self::Error> {
        Ok(IdentifiedAnyClientState {
            client_id: raw.client_id.parse().map_err(|e: ValidationError| {
                Error::invalid_raw_client_id(raw.client_id.clone(), e)
            })?,
            client_state: raw
                .client_state
                .ok_or_else(Error::missing_raw_client_state)?
                .try_into()?,
        })
    }
}

impl<G> From<IdentifiedAnyClientState<G>> for IdentifiedClientState {
    fn from(value: IdentifiedAnyClientState<G>) -> Self {
        IdentifiedClientState {
            client_id: value.client_id.to_string(),
            client_state: Some(value.client_state.into()),
        }
    }
}

#[cfg(test)]
mod tests {

    use ibc_proto::google::protobuf::Any;
    use test_log::test;

    use crate::clients::ics07_tendermint::client_state::test_util::get_dummy_tendermint_client_state;
    use crate::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;
    use crate::core::ics02_client::client_state::AnyClientState;

    #[test]
    fn any_client_state_serialization() {
        let tm_client_state = get_dummy_tendermint_client_state(get_dummy_tendermint_header());

        let raw: Any = tm_client_state.clone().into();
        let tm_client_state_back = AnyClientState::try_from(raw).unwrap();
        assert_eq!(tm_client_state, tm_client_state_back);
    }
}
