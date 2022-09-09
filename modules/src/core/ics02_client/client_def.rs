use crate::clients::host_functions::HostFunctionsProvider;
use crate::clients::ics07_tendermint::client_def::TendermintClient;
use crate::clients::ics07_tendermint::consensus_state::ConsensusState as TendermintConsensusState;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::{
    client_def::BeefyClient, consensus_state::ConsensusState as BeefyConsensusState,
};
use crate::clients::{ClientStateOf, ClientTypesOf, ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::ics02_client::client_type::{ClientType, ClientTypes};
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::header::AnyHeader;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics26_routing::context::ReaderContext;
use crate::downcast;
#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::MockConsensusState;
use crate::prelude::*;
use crate::Height;
use core::fmt::{Debug, Display};
use derivative::Derivative;

use ibc_proto::google::protobuf::Any;
use std::marker::PhantomData;
use tendermint_proto::Protobuf;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_def::MockClient;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ConsensusUpdateResult<C: ClientTypes> {
    Single(C::ConsensusState),
    Batch(Vec<(Height, C::ConsensusState)>),
}

impl<C: ClientTypes> ConsensusUpdateResult<C> {
    pub fn map_state<F, D: ClientTypes>(self, f: F) -> ConsensusUpdateResult<D>
    where
        F: Fn(C::ConsensusState) -> D::ConsensusState,
    {
        match self {
            ConsensusUpdateResult::Single(cs) => ConsensusUpdateResult::Single(f(cs)),
            ConsensusUpdateResult::Batch(cs) => {
                ConsensusUpdateResult::Batch(cs.into_iter().map(|(h, s)| (h, f(s))).collect())
            }
        }
    }
}

pub trait ClientDef: ClientTypes + Clone {
    type G: GlobalDefs;

    fn verify_header<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(), Error>;

    fn update_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx::ClientTypes>), Error>;

    fn update_state_on_misbehaviour(
        &self,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<Self::ClientState, Error>;

    fn check_for_misbehaviour<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<bool, Error>;

    /// TODO
    fn verify_upgrade_and_update_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx::ClientTypes>), Error>;

    /// Verification functions as specified in:
    /// <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics>
    ///
    /// Verify a `proof` that the consensus state of a given client (at height `consensus_height`)
    /// matches the input `consensus_state`. The parameter `counterparty_height` represent the
    /// height of the counterparty chain that this proof assumes (i.e., the height at which this
    /// proof was computed).
    #[allow(clippy::too_many_arguments)]
    fn verify_client_consensus_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        expected_consensus_state: &<Ctx::ClientTypes as ClientTypes>::ConsensusState,
    ) -> Result<(), Error>;

    /// Verify a `proof` that a connection state matches that of the input `connection_end`.
    #[allow(clippy::too_many_arguments)]
    fn verify_connection_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        connection_id: &ConnectionId,
        expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error>;

    /// Verify a `proof` that a channel state matches that of the input `channel_end`.
    #[allow(clippy::too_many_arguments)]
    fn verify_channel_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error>;

    /// Verify the client state for this chain that it is stored on the counterparty chain.
    #[allow(clippy::too_many_arguments)]
    fn verify_client_full_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        expected_client_state: &<Ctx::ClientTypes as ClientTypes>::ClientState,
    ) -> Result<(), Error>;

    /// Verify a `proof` that a packet has been commited.
    #[allow(clippy::too_many_arguments)]
    fn verify_packet_data<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), Error>;

    /// Verify a `proof` that a packet has been commited.
    #[allow(clippy::too_many_arguments)]
    fn verify_packet_acknowledgement<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        ack: AcknowledgementCommitment,
    ) -> Result<(), Error>;

    /// Verify a `proof` that of the next_seq_received.
    #[allow(clippy::too_many_arguments)]
    fn verify_next_sequence_recv<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Error>;

    /// Verify a `proof` that a packet has not been received.
    #[allow(clippy::too_many_arguments)]
    fn verify_packet_receipt_absence<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Error>;

    fn from_client_type(client_type: ClientType) -> Self;
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "G: Clone"),
    Debug(bound = "G: Debug"),
    PartialEq(bound = "G: PartialEq"),
    Eq(bound = "G: Eq")
)]
pub enum AnyClient<G: GlobalDefs>
where
    G::HostFunctions: HostFunctionsProvider + Clone + Debug + Eq,
{
    Tendermint(TendermintClient<G>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Beefy(BeefyClient<G>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Near(BeefyClient<G>),
    #[cfg(any(test, feature = "mocks"))]
    Mock(MockClient<G>),
}

impl<G: GlobalDefs> AnyClient<G>
where
    G::HostFunctions: HostFunctionsProvider + Clone + Debug + Eq + Default,
{
    pub fn from_client_type(client_type: ClientType) -> Self {
        match client_type {
            ClientType::Tendermint => Self::Tendermint(TendermintClient::<G>::default()),
            #[cfg(any(test, feature = "ics11_beefy"))]
            ClientType::Beefy => Self::Beefy(BeefyClient::<G>::default()),
            #[cfg(any(test, feature = "ics11_beefy"))]
            ClientType::Near => Self::Near(BeefyClient::<G>::default()),
            #[cfg(any(test, feature = "mocks"))]
            ClientType::Mock => Self::Mock(MockClient::default()),
        }
    }
}

impl<G: GlobalDefs + Clone> ClientTypes for AnyClient<G>
where
    G::HostFunctions: HostFunctionsProvider + Clone + Debug + Eq,

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
    type Header = AnyHeader;
    type ClientState = AnyClientState<G>;
    type ConsensusState = AnyConsensusState;
}

#[derive(Derivative)]
#[derivative(
    Default(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Debug(bound = "")
)]
pub struct AnyGlobalDef<H>(PhantomData<H>);

pub mod stub_beefy {
    pub struct Stub;
    pub type BeefyConsensusState = Stub;
}
#[cfg(not(feature = "ics11_beefy"))]
use stub_beefy::*;

#[cfg(not(test))]
pub mod stub_mock {
    pub struct Stub;
    pub type MockConsensusState = Stub;
}
#[cfg(not(test))]
use stub_mock::*;

// ⚠️  Beware of the awful boilerplate below ⚠️
impl<G: GlobalDefs + Clone> ClientDef for AnyClient<G>
where
    G::HostFunctions: HostFunctionsProvider + Clone + Debug + Eq,

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
    type G = G;

    /// Validate an incoming header
    fn verify_header<Ctx>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(), Error>
    where
        Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>,
    {
        match self {
            Self::Tendermint(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    header => AnyHeader::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_header::<Ctx>(ctx, client_id, client_state, header)
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Beefy,
                    header => AnyHeader::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_header::<Ctx>(ctx, client_id, client_state, header)
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Beefy,
                    header => AnyHeader::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_header(ctx, client_id, client_state, header)
            }

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Mock,
                    header => AnyHeader::Mock,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_header(ctx, client_id, client_state, header)
            }
        }
    }

    /// Validates an incoming `header` against the latest consensus state of this client.
    fn update_state<Ctx>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: AnyHeader,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx::ClientTypes>), Error>
    where
        Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>,
    {
        match self {
            Self::Tendermint(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    header => AnyHeader::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                let (new_state, new_consensus) =
                    client.update_state(ctx, client_id, client_state, header)?;

                Ok((AnyClientState::Tendermint(new_state), new_consensus))
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Beefy,
                    header => AnyHeader::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                let (new_state, new_consensus) =
                    client.update_state(ctx, client_id, client_state, header)?;

                Ok((AnyClientState::Beefy(new_state), new_consensus))
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Mock,
                    header => AnyHeader::Mock,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                let (new_state, new_consensus) =
                    client.update_state(ctx, client_id, client_state, header)?;

                Ok((AnyClientState::Mock(new_state), new_consensus))
            }
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<Self::ClientState, Error> {
        match self {
            AnyClient::Tendermint(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    header => AnyHeader::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;
                let client_state = client.update_state_on_misbehaviour(client_state, header)?;
                Ok(Self::ClientState::Tendermint(client_state))
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClient::Beefy(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Beefy,
                    header => AnyHeader::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                let client_state = client.update_state_on_misbehaviour(client_state, header)?;
                Ok(Self::ClientState::Beefy(client_state))
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClient::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyClient::Mock(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Mock,
                    header => AnyHeader::Mock,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                let client_state = client.update_state_on_misbehaviour(client_state, header)?;
                Ok(Self::ClientState::Mock(client_state))
            }
        }
    }

    /// Checks for misbehaviour in an incoming header
    fn check_for_misbehaviour<Ctx>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<bool, Error>
    where
        Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>,
    {
        match self {
            AnyClient::Tendermint(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    header => AnyHeader::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;
                client.check_for_misbehaviour(ctx, client_id, client_state, header)
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClient::Beefy(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Beefy,
                    header => AnyHeader::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.check_for_misbehaviour(ctx, client_id, client_state, header)
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            AnyClient::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            AnyClient::Mock(client) => {
                let (client_state, header) = downcast!(
                    client_state => AnyClientState::Mock,
                    header => AnyHeader::Mock,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.check_for_misbehaviour(ctx, client_id, client_state, header)
            }
        }
    }

    fn verify_upgrade_and_update_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx::ClientTypes>), Error> {
        match self {
            Self::Tendermint(client) => {
                let (client_state, consensus_state) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    consensus_state => AnyConsensusState::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state::<Ctx>(
                    client_state,
                    consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok((AnyClientState::Tendermint(new_state), new_consensus))
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let (client_state, consensus_state) = downcast!(
                    client_state => AnyClientState::Beefy,
                    consensus_state => AnyConsensusState::Beefy,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state::<Ctx>(
                    client_state,
                    consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok((AnyClientState::Beefy(new_state), new_consensus))
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let (client_state, consensus_state) = downcast!(
                    client_state => AnyClientState::Mock,
                    consensus_state => AnyConsensusState::Mock,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state::<Ctx>(
                    client_state,
                    consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok((AnyClientState::Mock(new_state), new_consensus))
            }
        }
    }

    fn verify_client_consensus_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        expected_consensus_state: &<Ctx::ClientTypes as ClientTypes>::ConsensusState,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_client_consensus_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    consensus_height,
                    expected_consensus_state,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_client_consensus_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    consensus_height,
                    expected_consensus_state,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_client_consensus_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    consensus_height,
                    expected_consensus_state,
                )
            }
        }
    }

    fn verify_connection_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        connection_id: &ConnectionId,
        expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(client_state => AnyClientState::Tendermint)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_connection_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    connection_id,
                    expected_connection_end,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(client_state => AnyClientState::Beefy)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_connection_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    connection_id,
                    expected_connection_end,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(client_state => AnyClientState::Mock)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_connection_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    connection_id,
                    expected_connection_end,
                )
            }
        }
    }

    fn verify_channel_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(client_state => AnyClientState::Tendermint)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_channel_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    expected_channel_end,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(client_state => AnyClientState::Beefy)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_channel_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    expected_channel_end,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(client_state => AnyClientState::Mock)
                    .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_channel_state(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    expected_channel_end,
                )
            }
        }
    }

    fn verify_client_full_state<Ctx: ReaderContext<ClientTypes = ClientTypesOf<Self::G>>>(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        client_state_on_counterparty: &<Ctx::ClientTypes as ClientTypes>::ClientState,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_client_full_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    client_state_on_counterparty,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_client_full_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    client_state_on_counterparty,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_client_full_state(
                    ctx,
                    client_state,
                    height,
                    prefix,
                    proof,
                    root,
                    client_id,
                    client_state_on_counterparty,
                )
            }
        }
    }

    fn verify_packet_data<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_packet_data(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    commitment,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_packet_data(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    commitment,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_packet_data(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    commitment,
                )
            }
        }
    }

    fn verify_packet_acknowledgement<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_packet_acknowledgement(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    ack_commitment,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_packet_acknowledgement(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    ack_commitment,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_packet_acknowledgement(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                    ack_commitment,
                )
            }
        }
    }
    fn verify_next_sequence_recv<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_next_sequence_recv(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_next_sequence_recv(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_next_sequence_recv(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }
        }
    }

    fn verify_packet_receipt_absence<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: &ClientId,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Error> {
        match self {
            Self::Tendermint(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Tendermint
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                client.verify_packet_receipt_absence(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }

            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Beefy(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Beefy
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))?;

                client.verify_packet_receipt_absence(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            Self::Near(_) => {
                todo!()
            }
            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(client) => {
                let client_state = downcast!(
                    client_state => AnyClientState::Mock
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Mock))?;

                client.verify_packet_receipt_absence(
                    ctx,
                    client_id,
                    client_state,
                    height,
                    connection_end,
                    proof,
                    root,
                    port_id,
                    channel_id,
                    sequence,
                )
            }
        }
    }

    fn from_client_type(client_type: ClientType) -> Self {
        Self::from_client_type(client_type)
    }
}

impl TryFrom<AnyConsensusState> for TendermintConsensusState {
    type Error = Error;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        downcast!(
            value => AnyConsensusState::Tendermint
        )
        .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))
    }
}

impl From<TendermintConsensusState> for AnyConsensusState {
    fn from(value: TendermintConsensusState) -> Self {
        Self::Tendermint(value)
    }
}

#[cfg(any(test, feature = "ics11_beefy"))]
mod beefy_impls {
    use super::*;
    use crate::clients::ics11_beefy::consensus_state::ConsensusState as BeefyConsensusState;

    impl TryFrom<AnyConsensusState> for BeefyConsensusState {
        type Error = Error;

        fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
            downcast!(
            value => AnyConsensusState::Beefy
            )
            .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))
        }
    }

    impl From<BeefyConsensusState> for AnyConsensusState {
        fn from(value: BeefyConsensusState) -> Self {
            Self::Beefy(value)
        }
    }
}
