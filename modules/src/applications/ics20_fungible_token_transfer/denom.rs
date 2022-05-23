use crate::core::ics04_channel::context::ChannelReader;
use core::str::FromStr;
use subtle_encoding::hex;

use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::prelude::*;
use serde::{Deserialize, Serialize};

use super::error::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Denom(pub String);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashedDenom(pub Vec<u8>);

impl Denom {
    pub fn derive_denom(port_id: &PortId, channel_id: &ChannelId, denom: &str) -> Denom {
        Self(format!("{}/{}/{}", port_id, channel_id, denom))
    }

    pub fn derive_base_denom(&self) -> Result<Self, Error> {
        // Base denom is the string after the first PortId/Channel pair
        let (.., remainder) = self.0.split_once('/').ok_or_else(Error::invalid_denom)?;
        let (.., base_denom) = remainder.split_once('/').ok_or_else(Error::invalid_denom)?;
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
        Ok(Denom(format!("ibc/{}", denom_hex)))
    }

    /// Returns the prefix for this trace
    pub fn has_prefix(denom: &str, prefix: &str) -> bool {
        denom.starts_with(prefix)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// get_denom_prefix returns the receiving denomination prefix
    pub fn get_denom_prefix(port_id: &PortId, channel_id: &ChannelId) -> String {
        format!("{}/{}/", port_id, channel_id)
    }
}

impl FromStr for Denom {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<String> for Denom {
    fn from(value: String) -> Self {
        Self(value)
    }
}
