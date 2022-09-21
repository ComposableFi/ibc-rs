use alloc::vec::Vec;
use crate::core::ics02_client::error::Error;

/// Abstract of consensus state update information
pub trait ClientMessage: Clone + core::fmt::Debug + Send + Sync {
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

	fn decode_from_vec(bytes: Vec<u8>) -> Result<Self, Error>;
}
