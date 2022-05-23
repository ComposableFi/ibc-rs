use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::primitives::{Coin, DenomTrace};
use crate::applications::ics20_fungible_token_transfer::HashedDenom;
use crate::core::ics04_channel::context::{ChannelKeeper, ChannelReader};
use crate::core::ics05_port::context::{PortKeeper, PortReader};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use alloc::string::String;
use core::str::FromStr;

pub trait Ics20Keeper:
    ChannelKeeper
    + PortKeeper
    + BankKeeper<AccountId = <Self as Ics20Keeper>::AccountId>
    + AccountReader<AccountId = <Self as Ics20Keeper>::AccountId>
{
    /// The account identifier type.
    type AccountId: Into<String>;

    /// Set channel escrow address
    fn set_channel_escrow_address(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), Error>;
    /// Sets a new {trace hash -> denom trace} pair to the store.
    fn set_denom_trace(&mut self, denom_trace: DenomTrace) -> Result<(), Error>;
}

pub trait Ics20Reader:
    ChannelReader
    + PortReader
    + AccountReader<AccountId = <Self as Ics20Reader>::AccountId>
    + BankReader<AccountId = <Self as Ics20Reader>::AccountId>
{
    /// The account identifier type.
    type AccountId: Into<String> + FromStr<Err = Error>;

    /// Returns true if sending is allowed in the module params
    fn is_send_enabled(&self) -> bool;
    /// Returns true if receiving is allowed in the module params
    fn is_receive_enabled(&self) -> bool;
    /// Sets and returns the escrow account id for a port and channel combination
    fn get_channel_escrow_address(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<<Self as Ics20Reader>::AccountId, Error>;
    /// Returns true if the store contains a `DenomTrace` entry for the specified `HashedDenom`.
    fn has_denom_trace(&self, hashed_denom: HashedDenom) -> bool;
    /// Gets the denom trace associated with the specified hash in the store.
    fn get_denom_trace(&self, denom_hash: HashedDenom) -> Option<DenomTrace>;
}

pub trait BankKeeper {
    /// The account identifier type.
    type AccountId: Into<String> + FromStr<Err = Error>;

    /// This function should enable sending ibc fungible tokens from one account to another
    fn send_coins(
        &mut self,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amt: &Coin,
    ) -> Result<(), Error>;
    /// This function to enable  minting tokens(vouchers) in a module
    fn mint_coins(&mut self, amt: &Coin) -> Result<(), Error>;
    /// This function should enable burning of minted tokens or vouchers
    fn burn_coins(&mut self, module: &Self::AccountId, amt: &Coin) -> Result<(), Error>;
}

pub trait BankReader {
    /// The account identifier type.
    type AccountId: Into<String> + FromStr<Err = Error>;

    /// Returns true if the specified account is not allowed to receive funds and false otherwise.
    fn is_blocked_account(&self, account: &Self::AccountId) -> bool;
}

pub trait AccountReader {
    /// The account identifier type.
    type AccountId: Into<String> + FromStr<Err = Error>;

    /// This function should return the account of the ibc module
    fn get_module_account(&self) -> Self::AccountId;
}

pub trait Ics20Context:
    Ics20Keeper<AccountId = <Self as Ics20Context>::AccountId>
    + Ics20Reader<AccountId = <Self as Ics20Context>::AccountId>
{
    type AccountId: Into<String> + FromStr<Err = Error>;
}
