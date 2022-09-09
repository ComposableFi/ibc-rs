//! Definition of domain type message `MsgCreateAnyClient`.

use crate::prelude::*;
use core::fmt::Display;

use ibc_proto::google::protobuf::Any;
use tendermint_proto::Protobuf;

use ibc_proto::ibc::core::client::v1::{MsgCreateClient as RawMsgCreateClient, MsgCreateClient};

use crate::core::ics02_client::client_consensus::ConsensusState;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientTypes;
use crate::core::ics02_client::error::Error;
use crate::signer::Signer;
use crate::tx_msg::Msg;

pub const TYPE_URL: &str = "/ibc.core.client.v1.MsgCreateClient";

/// A type of message that triggers the creation of a new on-chain (IBC) client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgCreateAnyClient<C: ClientTypes> {
    pub client_state: C::ClientState,
    pub consensus_state: C::ConsensusState,
    pub signer: Signer,
}

impl<C: ClientTypes> MsgCreateAnyClient<C> {
    pub fn new(
        client_state: C::ClientState,
        consensus_state: C::ConsensusState,
        signer: Signer,
    ) -> Result<Self, Error> {
        if client_state.client_type() != consensus_state.client_type() {
            return Err(Error::raw_client_and_consensus_state_types_mismatch(
                client_state.client_type(),
                consensus_state.client_type(),
            ));
        }

        Ok(MsgCreateAnyClient {
            client_state,
            consensus_state,
            signer,
        })
    }
}

impl<C> Msg for MsgCreateAnyClient<C>
where
    C: ClientTypes + Clone,
    Any: From<C::ClientState>,
    Any: From<C::ConsensusState>,
{
    type ValidationError = crate::core::ics24_host::error::ValidationError;
    type Raw = RawMsgCreateClient;

    fn route(&self) -> String {
        crate::keys::ROUTER_KEY.to_string()
    }

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl<C> Protobuf<RawMsgCreateClient> for MsgCreateAnyClient<C>
where
    C: ClientTypes + Clone,
    Any: From<C::ClientState>,
    Any: From<C::ConsensusState>,
    MsgCreateAnyClient<C>: TryFrom<MsgCreateClient>,
    <MsgCreateAnyClient<C> as TryFrom<MsgCreateClient>>::Error: Display,
{
}

impl<C> TryFrom<RawMsgCreateClient> for MsgCreateAnyClient<C>
where
    C: ClientTypes,
    C::ClientState: TryFrom<Any>,
    C::ConsensusState: TryFrom<Any>,
    Error: From<<C::ClientState as TryFrom<Any>>::Error>,
{
    type Error = Error;

    fn try_from(raw: RawMsgCreateClient) -> Result<Self, Error> {
        let raw_client_state = raw
            .client_state
            .ok_or_else(Error::missing_raw_client_state)?;

        let consensus_state = raw
            .consensus_state
            .and_then(|cs| C::ConsensusState::try_from(cs).ok())
            .ok_or_else(Error::missing_raw_consensus_state)?;

        MsgCreateAnyClient::new(
            C::ClientState::try_from(raw_client_state)?,
            consensus_state,
            raw.signer.parse().map_err(Error::signer)?,
        )
    }
}

impl<C> From<MsgCreateAnyClient<C>> for RawMsgCreateClient
where
    C: ClientTypes,
    Any: From<C::ClientState>,
    Any: From<C::ConsensusState>,
{
    fn from(ics_msg: MsgCreateAnyClient<C>) -> Self {
        RawMsgCreateClient {
            client_state: Some(ics_msg.client_state.into()),
            consensus_state: Some(ics_msg.consensus_state.into()),
            signer: ics_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use test_log::test;

    use crate::clients::ClientTypesOf;
    use ibc_proto::ibc::core::client::v1::MsgCreateClient;

    use crate::clients::ics07_tendermint::client_state::test_util::get_dummy_tendermint_client_state;
    use crate::clients::ics07_tendermint::header::test_util::get_dummy_tendermint_header;
    use crate::core::ics02_client::client_consensus::AnyConsensusState;
    use crate::core::ics02_client::msgs::MsgCreateAnyClient;
    use crate::mock::client_def::TestGlobalDefs;
    use crate::test_utils::get_dummy_account_id;

    #[test]
    fn msg_create_client_serialization() {
        let signer = get_dummy_account_id();

        let tm_header = get_dummy_tendermint_header();
        let tm_client_state = get_dummy_tendermint_client_state(tm_header.clone());

        let msg = MsgCreateAnyClient::<ClientTypesOf<TestGlobalDefs>>::new(
            tm_client_state,
            AnyConsensusState::Tendermint(tm_header.try_into().unwrap()),
            signer,
        )
        .unwrap();

        let raw = MsgCreateClient::from(msg.clone());
        let msg_back = MsgCreateAnyClient::try_from(raw.clone()).unwrap();
        let raw_back = MsgCreateClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
