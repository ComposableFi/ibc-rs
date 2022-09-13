use core::fmt::{Debug, Display};
use core::marker::{Send, Sync};
use core::time::Duration;

use ibc_proto::google::protobuf::Any;
use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use crate::clients::ics07_tendermint::client_state;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::client_state as beefy_client_state;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::Error;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::MockClientState;
use crate::prelude::*;
use crate::{downcast, Height};
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;

pub const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";
pub const BEEFY_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.beefy.v1.ClientState";
pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

pub trait ClientState: Clone + Debug + Send + Sync {
    /// Client-specific options for upgrading the client
    type UpgradeOptions;

    /// Return the chain identifier which this client is serving (i.e., the client is verifying
    /// consensus states from this chain).
    fn chain_id(&self) -> ChainId;

    /// Type of client associated with this state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

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

    /// Performs downcast of the client state from an "AnyClientState" type to T, otherwise
    /// panics. Downcast from `T` to `T` is always successful.
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
            .expect("ClientState wrap failed")
            .clone()
    }

    fn encode_to_vec(&self) -> Vec<u8>;
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

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, derive::ClientState, derive::Protobuf,
)]
#[serde(tag = "type")]
pub enum AnyClientState {
    #[ibc(proto_url = "TENDERMINT_CLIENT_STATE_TYPE_URL")]
    Tendermint(client_state::ClientState),
    #[cfg(any(test, feature = "ics11_beefy"))]
    #[serde(skip)]
    #[ibc(proto_url = "BEEFY_CLIENT_STATE_TYPE_URL")]
    Beefy(beefy_client_state::ClientState),
    // #[cfg(any(test, feature = "ics11_beefy"))]
    // #[serde(skip)]
    // Near(beefy_client_state::ClientState),
    #[cfg(any(test, feature = "mocks"))]
    #[ibc(proto_url = "MOCK_CLIENT_STATE_TYPE_URL")]
    Mock(MockClientState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub struct IdentifiedAnyClientState {
    pub client_id: ClientId,
    pub client_state: AnyClientState,
}

impl IdentifiedAnyClientState {
    pub fn new(client_id: ClientId, client_state: AnyClientState) -> Self {
        IdentifiedAnyClientState {
            client_id,
            client_state,
        }
    }
}

impl Protobuf<IdentifiedClientState> for IdentifiedAnyClientState
where
    IdentifiedAnyClientState: TryFrom<IdentifiedClientState>,
    <IdentifiedAnyClientState as TryFrom<IdentifiedClientState>>::Error: Display,
{
}

impl TryFrom<IdentifiedClientState> for IdentifiedAnyClientState
where
    AnyClientState: TryFrom<Any>,
    Error: From<<AnyClientState as TryFrom<Any>>::Error>,
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

impl From<IdentifiedAnyClientState> for IdentifiedClientState {
    fn from(value: IdentifiedAnyClientState) -> Self {
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
