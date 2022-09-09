//! Implementations of client verification algorithms for specific types of chains.

use crate::clients::host_functions::HostFunctionsProvider;
use crate::core::ics02_client::client_def::ClientDef;
use crate::core::ics02_client::client_type::ClientTypes;

use core::fmt::Debug;

pub mod host_functions;
pub mod ics07_tendermint;
#[cfg(any(test, feature = "ics11_beefy"))]
pub mod ics11_beefy;
#[cfg(any(test, feature = "ics11_beefy"))]
pub mod ics13_near;

pub trait GlobalDefs: Send + Sync {
    type HostFunctions: HostFunctionsProvider;
    type ClientTypes: ClientTypes + Debug + Eq + Clone;
    // todo: use GATs to make this not the business of this trait once it's stable.
    type ClientDef: ClientDef<
            G = Self,
            ClientState = <Self::ClientTypes as ClientTypes>::ClientState,
            ConsensusState = <Self::ClientTypes as ClientTypes>::ConsensusState,
            Header = <Self::ClientTypes as ClientTypes>::Header,
        > + Debug
        + Eq;
}

pub type ConsensusStateOf<G> = <ClientTypesOf<G> as ClientTypes>::ConsensusState;
pub type ClientStateOf<G> = <ClientTypesOf<G> as ClientTypes>::ClientState;
pub type ClientTypesOf<G> = <G as GlobalDefs>::ClientTypes;
