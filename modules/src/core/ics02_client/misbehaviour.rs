use crate::core::ics24_host::identifier::ClientId;
use crate::prelude::*;
use crate::Height;

pub trait Misbehaviour: Clone + core::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_id(&self) -> &ClientId;

    /// The height of the consensus state
    fn height(&self) -> Height;

    fn downcast<T: Clone + 'static>(self) -> T
    where
        Self: 'static,
    {
        <dyn core::any::Any>::downcast_ref(&self)
            .cloned()
            .expect("Header downcast failed")
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Self
    where
        Self: 'static,
    {
        sub_state
            .downcast_ref::<Self>()
            .expect("Header wrap failed")
            .clone()
    }

    fn encode_to_vec(&self) -> Vec<u8>;
}
