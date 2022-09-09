use crate::clients::{ClientTypesOf, ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_def::{AnyClient, ClientDef, ConsensusUpdateResult};
use crate::core::ics02_client::client_type::{ClientType, ClientTypes};
use crate::core::ics02_client::error::Error;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics23_commitment::merkle::apply_prefix;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::ics24_host::Path;
use crate::core::ics26_routing::context::ReaderContext;
use crate::mock::client_state::{MockClientState, MockConsensusState};
use crate::mock::context::MockTypes;
use crate::mock::header::MockHeader;
use crate::prelude::*;
use crate::test_utils::Crypto;
use crate::Height;
use core::fmt::Debug;
use derivative::Derivative;
use std::marker::PhantomData;

#[derive(Derivative, Debug, PartialEq, Eq)]
#[derivative(Clone(bound = ""))]
pub struct MockClient<G>(PhantomData<G>);

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TestGlobalDefs;
impl GlobalDefs for TestGlobalDefs {
    type HostFunctions = Crypto;
    type ClientTypes = MockTypes;
    type ClientDef = AnyClient<TestGlobalDefs>;
}

pub type TestMockClient = MockClient<TestGlobalDefs>;

impl<G> Default for MockClient<G> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl<G: GlobalDefs + Clone> ClientTypes for MockClient<G>
where
    MockConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<MockConsensusState>,
{
    type Header = MockHeader;
    type ClientState = MockClientState<G>;
    type ConsensusState = MockConsensusState;
}

impl<G> ClientDef for MockClient<G>
where
    MockConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<MockConsensusState>,

    G: GlobalDefs + Clone,
{
    type G = G;

    fn update_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        _ctx: &Ctx,
        _client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<
        (
            Self::ClientState,
            ConsensusUpdateResult<<Ctx as ReaderContext>::ClientTypes>,
        ),
        Error,
    > {
        if client_state.latest_height() >= header.height() {
            return Err(Error::low_header_height(
                header.height(),
                client_state.latest_height(),
            ));
        }

        Ok((
            MockClientState::new(header),
            ConsensusUpdateResult::Single(MockConsensusState::new(header).into()),
        ))
    }

    fn verify_client_consensus_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        _ctx: &Ctx,
        _client_state: &Self::ClientState,
        _height: Height,
        prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        _expected_consensus_state: &Ctx::ConsensusState,
    ) -> Result<(), Error> {
        let client_prefixed_path = Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: consensus_height.revision_number,
            height: consensus_height.revision_height,
        })
        .to_string();

        let _path = apply_prefix(prefix, vec![client_prefixed_path]);

        Ok(())
    }

    fn verify_connection_state<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _connection_id: &ConnectionId,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_channel_state<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_client_full_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        _ctx: &Ctx,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _expected_client_state: &Ctx::ClientState,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_data<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _commitment: PacketCommitment,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_acknowledgement<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _ack: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_next_sequence_recv<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_packet_receipt_absence<Ctx: ReaderContext>(
        &self,
        _ctx: &Ctx,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn verify_upgrade_and_update_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<G>>>(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<
        (
            Self::ClientState,
            ConsensusUpdateResult<<Ctx as ReaderContext>::ClientTypes>,
        ),
        Error,
    > {
        Ok((
            *client_state,
            ConsensusUpdateResult::Single(consensus_state.clone().into()),
        ))
    }

    fn verify_header<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        _ctx: &Ctx,
        _client_id: ClientId,
        _client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn update_state_on_misbehaviour(
        &self,
        client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<Self::ClientState, Error> {
        Ok(client_state)
    }

    fn check_for_misbehaviour<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        _ctx: &Ctx,
        _client_id: ClientId,
        _client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    fn from_client_type(_client_type: ClientType) -> Self {
        todo!()
    }
}
