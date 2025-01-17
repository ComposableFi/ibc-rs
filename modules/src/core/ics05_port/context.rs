use crate::{
	core::{
		ics05_port::error::Error, ics24_host::identifier::PortId, ics26_routing::context::ModuleId,
	},
	prelude::*,
};

/// A context supplying all the necessary read-only dependencies for processing any information
/// regarding a port.
pub trait PortReader {
	/// Return the module_id associated with a given port_id
	fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, Error>;
}
