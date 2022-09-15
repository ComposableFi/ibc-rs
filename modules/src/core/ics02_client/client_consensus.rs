use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::ics24_host::identifier::ClientId;
use crate::events::WithBlockDataType;
use crate::prelude::*;
use crate::timestamp::Timestamp;
use core::fmt::Debug;
use core::marker::{Send, Sync};

pub trait ConsensusState: Clone + Debug + Send + Sync {
    type Error;

    /// Type of client associated with this consensus state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Commitment root of the consensus state, which is used for key-value pair verification.
    fn root(&self) -> &CommitmentRoot;

    /// Returns the timestamp of the state.
    fn timestamp(&self) -> Timestamp;

    fn downcast<T: Clone + 'static>(self) -> Option<T>
    where
        Self: 'static,
    {
        <dyn core::any::Any>::downcast_ref(&self).cloned()
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Option<Self>
    where
        Self: 'static,
    {
        sub_state.downcast_ref::<Self>().cloned()
    }

    fn encode_to_vec(&self) -> Vec<u8>;
}

/// Query request for a single client event, identified by `event_id`, for `client_id`.
#[derive(Clone, Debug)]
pub struct QueryClientEventRequest {
    pub height: crate::Height,
    pub event_id: WithBlockDataType,
    pub client_id: ClientId,
    pub consensus_height: crate::Height,
}
