use subtle_encoding::hex;
use crate::core::ics04_channel::context::ChannelReader;

use serde::{Serialize, Deserialize};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::prelude::*;

use super::error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Denom(String);

impl Denom {
    pub fn new(raw_str: String) -> Self {
        Self(raw_str)
    }

    pub fn derive_denom(port_id: &PortId, channel_id: &ChannelId, denom: &str,) -> Denom {
        Self(format!("{}/{}/{}", port_id, channel_id, denom))
    }

    pub fn derive_base_denom(&self) -> Result<Self, Error> {
        // Base denom is the string after the first PortId/Channel pair
        let (.., remainder) = self.0.split_once('/').ok_or_else(|| Error::invalid_denom())?;
        let (.., base_denom) = remainder.split_once('/').ok_or_else(|| Error::invalid_denom())?;
        Ok(Self(base_denom.to_string()))
    }

    /// Derive the transferred token denomination using
    /// <https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md>
    pub fn derive_ibc_denom(
        ctx: &dyn ChannelReader,
        port_id: &PortId,
        channel_id: &ChannelId,
        denom: &str,
    ) -> Result<Denom, Error> {
        let transfer_path = format!("{}/{}/{}", port_id, channel_id, denom);
        let denom_bytes = ctx.hash(transfer_path.as_bytes().to_vec());
        let denom_hex = String::from_utf8(hex::encode_upper(denom_bytes)).map_err(Error::utf8)?;
        Ok(Denom(format!("ibc/{}", denom_hex))
    }
}
