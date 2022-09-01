use crate::clients::host_functions::HostFunctionsProvider;
use crate::clients::ics07_tendermint::client_def::TendermintClient;
use crate::clients::ics07_tendermint::client_state::ClientState as TendermintClientState;
#[cfg(any(test, feature = "ics11_beefy"))]
use crate::clients::ics11_beefy::client_def::BeefyClient;
use crate::clients::{ClientStateOf, ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::core::ics02_client::client_state::{AnyClientState, ClientState};
use crate::core::ics02_client::client_type::{ClientType, ClientTypes};
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::header::{AnyHeader, Header};
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
use crate::prelude::*;
use crate::Height;
use core::fmt::{Debug, Display, Formatter};
use derivative::Derivative;
use ibc_proto::google::protobuf::Any;
use std::convert::Infallible;
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

    fn verify_header<Ctx: ReaderContext<ClientTypes = <Self::G as GlobalDefs>::ClientDef>>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(), Error>
    where
        ConsensusStateOf<Self::G>: From<Ctx::ConsensusState>,
        Ctx::ConsensusState: From<ConsensusStateOf<Self::G>>;

    fn update_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error>
    where
        Ctx::ConsensusState: From<ConsensusStateOf<Self::G>>;

    fn update_state_on_misbehaviour(
        &self,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<Self::ClientState, Error>;

    fn check_for_misbehaviour<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<bool, Error>
    where
        ConsensusStateOf<Self::G>: From<Ctx::ConsensusState>;

    /// TODO
    fn verify_upgrade_and_update_state<Ctx: ReaderContext>(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error>; // this is where the issue is

    /// Verification functions as specified in:
    /// <https://github.com/cosmos/ibc/tree/master/spec/core/ics-002-client-semantics>
    ///
    /// Verify a `proof` that the consensus state of a given client (at height `consensus_height`)
    /// matches the input `consensus_state`. The parameter `counterparty_height` represent the
    /// height of the counterparty chain that this proof assumes (i.e., the height at which this
    /// proof was computed).
    #[allow(clippy::too_many_arguments)]
    fn verify_client_consensus_state<
        Ctx: ReaderContext<ClientTypes = <Self::G as GlobalDefs>::ClientDef>,
    >(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        expected_consensus_state: &Ctx::ConsensusState,
    ) -> Result<(), Error>
where
        // ConsensusStateOf<Self::G>: From<Any>,
        // ConsensusStateOf<Self::G>: Protobuf<Any>,
        // Any: From<ConsensusStateOf<Self::G>>,
        // ConsensusStateOf<Self::G>: TryFrom<Any>,
        // <ConsensusStateOf<Self::G> as TryFrom<Any>>::Error: Display
    ;

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
    fn verify_client_full_state<
        Ctx: ReaderContext<ClientTypes = <Self::G as GlobalDefs>::ClientDef>,
    >(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        expected_client_state: &Ctx::ClientState,
    ) -> Result<(), Error>
where
        // ClientStateOf<Self::G>: Protobuf<Any>
        //     Any: From<ClientStateOf<Self::G>>,
        //     ClientStateOf<Self::G>: TryFrom<Any>,
        //     <ClientStateOf<Self::G> as TryFrom<Any>>::Error: Display
    ;

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
    Clone(bound = "AnyGlobalDef<H>: Clone"),
    Debug(bound = "AnyGlobalDef<H>: Debug"),
    PartialEq(bound = "AnyGlobalDef<H>: PartialEq"),
    Eq(bound = "AnyGlobalDef<H>: Eq")
)]
pub enum AnyClient<H>
where
    H: HostFunctionsProvider + Clone + Debug + Eq,
{
    Tendermint(TendermintClient<AnyGlobalDef<H>>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Beefy(BeefyClient<AnyGlobalDef<H>>),
    #[cfg(any(test, feature = "ics11_beefy"))]
    Near(BeefyClient<AnyGlobalDef<H>>),
    #[cfg(any(test, feature = "mocks"))]
    Mock(MockClient),
}

impl<H> AnyClient<H>
where
    H: HostFunctionsProvider + Clone + Debug + Eq + Default,
{
    pub fn from_client_type(client_type: ClientType) -> Self {
        match client_type {
            ClientType::Tendermint => {
                Self::Tendermint(TendermintClient::<AnyGlobalDef<H>>::default())
            }
            #[cfg(any(test, feature = "ics11_beefy"))]
            ClientType::Beefy => Self::Beefy(BeefyClient::<AnyGlobalDef<H>>::default()),
            #[cfg(any(test, feature = "ics11_beefy"))]
            ClientType::Near => Self::Near(BeefyClient::<AnyGlobalDef<H>>::default()),
            #[cfg(any(test, feature = "mocks"))]
            ClientType::Mock => Self::Mock(MockClient::default()),
        }
    }
}

impl<H> ClientTypes for AnyClient<H>
where
    H: HostFunctionsProvider + Clone + Debug + Eq,
{
    type Header = AnyHeader;
    type ClientState = AnyClientState<AnyGlobalDef<H>>;
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

impl<H: HostFunctionsProvider> GlobalDefs for AnyGlobalDef<H>
where
    H: Send + Sync + Clone + Debug + Eq,
{
    type HostFunctions = H;
    type ClientDef = AnyClient<H>;
}

impl<H> TryFrom<crate::clients::ics07_tendermint::client_state::ClientState<AnyGlobalDef<H>>>
    for AnyClientState<AnyGlobalDef<H>>
where
    H: HostFunctionsProvider + Clone + Debug + Eq,
{
    type Error = Error;

    fn try_from(value: TendermintClientState<AnyGlobalDef<H>>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<AnyConsensusState>
    for crate::clients::ics07_tendermint::consensus_state::ConsensusState
{
    type Error = Error;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        downcast!(
            value => AnyConsensusState::Tendermint
        )
        .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))
    }
}

impl From<crate::clients::ics07_tendermint::consensus_state::ConsensusState> for AnyConsensusState {
    fn from(_: crate::clients::ics07_tendermint::consensus_state::ConsensusState) -> Self {
        todo!()
    }
}

impl<H> TryFrom<crate::clients::ics11_beefy::client_state::ClientState<AnyGlobalDef<H>>>
    for AnyClientState<AnyGlobalDef<H>>
where
    H: HostFunctionsProvider + Clone + Debug + Eq,
{
    type Error = Error;

    fn try_from(
        value: crate::clients::ics11_beefy::client_state::ClientState<AnyGlobalDef<H>>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<AnyConsensusState> for crate::clients::ics11_beefy::consensus_state::ConsensusState {
    type Error = Error;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        downcast!(
            value => AnyConsensusState::Beefy
        )
        .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Beefy))
    }
}

impl From<crate::clients::ics11_beefy::consensus_state::ConsensusState> for AnyConsensusState {
    fn from(_: crate::clients::ics11_beefy::consensus_state::ConsensusState) -> Self {
        todo!()
    }
}

// ⚠️  Beware of the awful boilerplate below ⚠️
impl<H> ClientDef for AnyClient<H>
where
    H: HostFunctionsProvider + Clone + Debug + Eq,
{
    type G = AnyGlobalDef<H>;

    /// Validate an incoming header
    fn verify_header<Ctx>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
        // trusted_consensus_state: Self::ConsensusState,
    ) -> Result<(), Error>
    where
        Ctx: ReaderContext<ClientTypes = Self>,
        ConsensusStateOf<Self::G>: From<Ctx::ConsensusState>,
        Ctx::ConsensusState: From<ConsensusStateOf<Self::G>>,
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
    fn update_state<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: AnyHeader,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error>
    where
        Ctx::ConsensusState: From<ConsensusStateOf<Self::G>>,
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
    fn check_for_misbehaviour<Ctx: ReaderContext>(
        &self,
        ctx: &Ctx,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<bool, Error>
    where
        ConsensusStateOf<Self::G>: From<Ctx::ConsensusState>,
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

    fn verify_upgrade_and_update_state<Ctx: ReaderContext>(
        &self,
        client_state: &Self::ClientState,
        consensus_state: &Self::ConsensusState,
        proof_upgrade_client: Vec<u8>,
        proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult<Ctx>), Error> {
        match self {
            Self::Tendermint(client) => {
                let (client_state, consensus_state) = downcast!(
                    client_state => AnyClientState::Tendermint,
                    consensus_state => AnyConsensusState::Tendermint,
                )
                .ok_or_else(|| Error::client_args_type_mismatch(ClientType::Tendermint))?;

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state(
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

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state(
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

                let (new_state, new_consensus) = client.verify_upgrade_and_update_state(
                    client_state,
                    consensus_state,
                    proof_upgrade_client,
                    proof_upgrade_consensus_state,
                )?;

                Ok((AnyClientState::Mock(new_state), new_consensus))
            }
        }
    }

    fn verify_client_consensus_state<
        Ctx: ReaderContext<ClientTypes = <Self::G as GlobalDefs>::ClientDef>,
    >(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        expected_consensus_state: &Ctx::ConsensusState,
    ) -> Result<(), Error>
where
        // ConsensusStateOf<Self::G>: Protobuf<Any>,
        // Any: From<ConsensusStateOf<Self::G>>,
        // ConsensusStateOf<Self::G>: TryFrom<Any>,
        // <ConsensusStateOf<Self::G> as TryFrom<Any>>::Error: Display,
    {
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
    fn verify_client_full_state<Ctx: ReaderContext<ClientTypes = Self>>(
        &self,
        ctx: &Ctx,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        client_state_on_counterparty: &Ctx::ClientState,
    ) -> Result<(), Error>
where
        // Ctx::ClientState: Protobuf<Any>,
        // Any: From<Ctx::ClientState>,
        // Ctx::ClientState: TryFrom<Any>,
        // <Ctx::ClientState as TryFrom<Any>>::Error: Display,
    {
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
        todo!()
    }
}
