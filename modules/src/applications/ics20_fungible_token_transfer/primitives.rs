use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::Denom;
use crate::prelude::*;
use alloc::string::String;
use ibc_proto::cosmos::base::v1beta1::Coin as RawCoin;
use ibc_proto::ibc::applications::transfer::v1::DenomTrace as RawDenomTrace;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct DenomTrace {
    /// path defines the chain of port/channel identifiers used for tracing the
    /// source of the fungible token.
    pub path: String,
    /// base denomination of the relayed fungible token.
    pub base_denom: String,
}

impl Protobuf<RawDenomTrace> for DenomTrace {}

impl TryFrom<RawDenomTrace> for DenomTrace {
    type Error = Error;

    fn try_from(raw: RawDenomTrace) -> Result<Self, Self::Error> {
        Ok(Self {
            path: raw.path,
            base_denom: raw.base_denom,
        })
    }
}

impl From<DenomTrace> for RawDenomTrace {
    fn from(value: DenomTrace) -> Self {
        Self {
            path: value.path,
            base_denom: value.base_denom,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Coin {
    /// Denomination
    pub denom: Denom,
    /// Amount
    pub amount: U256,
}

impl Protobuf<RawCoin> for Coin {}

impl TryFrom<RawCoin> for Coin {
    type Error = Error;

    fn try_from(raw: RawCoin) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: Denom(raw.denom),
            amount: U256::from_str_radix(raw.amount.as_str(), 16)
                .map_err(|_| Error::invalid_amount())?,
        })
    }
}

impl From<Coin> for RawCoin {
    fn from(value: Coin) -> Self {
        Self {
            denom: value.denom.0,
            amount: serde_json::to_string(&value.amount).unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FungibleTokenPacketData {
    /// Token denomination
    pub denomination: Denom,
    /// Amount to be sent
    pub amount: U256,
    /// Sender account
    pub sender: String,
    /// Receiver account
    pub receiver: String,
}

impl FungibleTokenPacketData {
    /// Convert to json bytes
    pub fn get_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}
