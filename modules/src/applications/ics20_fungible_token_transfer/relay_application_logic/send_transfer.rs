use crate::applications::ics20_fungible_token_transfer::context::Ics20Context;
use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::msgs::transfer::MsgTransfer;
use crate::applications::ics20_fungible_token_transfer::primitives::FungibleTokenPacketData;
use crate::applications::ics20_fungible_token_transfer::utils::{get_source_chain, SourceChain};
use crate::applications::ics20_fungible_token_transfer::Denom;
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::prelude::*;
use crate::timestamp::Timestamp;
use crate::Height;
use alloc::str::FromStr;

#[derive(Clone, Debug)]
pub struct SendTransferPacket {
    pub data: Vec<u8>,
    pub source_port: PortId,
    pub source_channel: ChannelId,
    pub destination_port: PortId,
    pub destination_channel: ChannelId,
    pub sequence: Sequence,
    /// Timeout height offset from the latest height for the channel client
    pub timeout_offset_height: Height,
    /// Timeout timestamp offset from the current timestamp of the channel client
    pub timeout_offset_timestamp: Timestamp,
}

pub fn send_transfer<Ctx>(ctx: &mut Ctx, msg: MsgTransfer) -> Result<SendTransferPacket, Error>
where
    Ctx: Ics20Context,
{
    if !ctx.is_send_enabled() {
        return Err(Error::send_disabled());
    }

    let source_channel_end = ctx
        .channel_end(&(msg.source_port.clone(), msg.source_channel.clone()))
        .map_err(Error::ics04_channel)?;

    let destination_port = source_channel_end.counterparty().port_id().clone();
    let destination_channel = source_channel_end
        .counterparty()
        .channel_id()
        .ok_or_else(|| {
            Error::destination_channel_not_found(
                msg.source_port.clone(),
                msg.source_channel.clone(),
            )
        })?;

    // get the next sequence
    let sequence = ctx
        .get_next_sequence_send(&(msg.source_port.clone(), msg.source_channel.clone()))
        .map_err(Error::ics04_channel)?;

    let full_denom_path = msg.token.denom.as_str();

    let sender = FromStr::from_str(msg.sender.as_str())?;
    match get_source_chain(&msg.source_port, &msg.source_channel, full_denom_path) {
        SourceChain::Sender => {
            let escrow_address =
                ctx.get_channel_escrow_address(&msg.source_port, &msg.source_channel)?;

            // Send tokens from sender to channel escrow
            ctx.send_coins(&sender, &escrow_address, &msg.token)?;
        }
        SourceChain::Receiver => {
            let module_acc = ctx.get_module_account();
            // Send tokens to module
            ctx.send_coins(&sender, &module_acc, &msg.token)?;
            // Burn tokens
            ctx.burn_coins(&module_acc, &msg.token)?;
        }
    }

    let data = FungibleTokenPacketData {
        denomination: Denom(full_denom_path.to_string()),
        amount: msg.token.amount,
        sender: msg.sender.as_str().to_string(),
        receiver: msg.receiver.as_str().to_string(),
    }
    .get_bytes();

    Ok(SendTransferPacket {
        data,
        source_port: msg.source_port,
        source_channel: msg.source_channel,
        destination_port,
        destination_channel: destination_channel.clone(),
        sequence,
        timeout_offset_height: msg.timeout_height,
        timeout_offset_timestamp: msg.timeout_timestamp,
    })
}
