use crate::core::ics02_client::client_type::ClientType;
use crate::prelude::*;
use crate::Height;

/// Abstract of consensus state update information
pub trait Header: Clone + core::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_type(&self) -> ClientType;

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

    /// The height of the header
    fn height(&self) -> Height;
}
