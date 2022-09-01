//! Implementations of client verification algorithms for specific types of chains.

use crate::clients::host_functions::HostFunctionsProvider;
use crate::core::ics02_client::client_def::ClientDef;
use crate::core::ics02_client::client_type::ClientTypes;
use beefy_client_primitives::HostFunctions;
use core::fmt::Debug;

pub mod host_functions;
pub mod ics07_tendermint;
#[cfg(any(test, feature = "ics11_beefy"))]
pub mod ics11_beefy;
#[cfg(any(test, feature = "ics11_beefy"))]
pub mod ics13_near;

pub trait GlobalDefs: Send + Sync {
    type HostFunctions: HostFunctionsProvider;
    type ClientDef: ClientDef<G = Self> + Debug + Eq;
}

pub type ConsensusStateOf<G> = <<G as GlobalDefs>::ClientDef as ClientTypes>::ConsensusState;
pub type ClientStateOf<G> = <<G as GlobalDefs>::ClientDef as ClientTypes>::ClientState;
