use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics24_host::identifier::ChainId;
use crate::prelude::*;
use crate::Height;
use core::fmt::Debug;
use core::marker::{Send, Sync};
use core::time::Duration;

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
