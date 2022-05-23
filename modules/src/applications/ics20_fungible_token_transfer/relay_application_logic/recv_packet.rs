use crate::applications::ics20_fungible_token_transfer::acknowledgement::ICS20Acknowledgement;
use crate::applications::ics20_fungible_token_transfer::context::Ics20Context;
use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::primitives::{
    Coin, FungibleTokenPacketData,
};
use crate::applications::ics20_fungible_token_transfer::utils::{get_source_chain, SourceChain};
use crate::applications::ics20_fungible_token_transfer::Denom;
use crate::core::ics04_channel::packet::Packet;
use alloc::str::FromStr;
use alloc::string::ToString;

/// Handles incoming packets with ICS20 data
/// To be called inside the on_recv_packet callback
pub fn on_recv_packet<Ctx>(
    ctx: &mut Ctx,
    packet: &Packet,
    data: &FungibleTokenPacketData,
) -> Result<ICS20Acknowledgement, Error>
where
    Ctx: Ics20Context,
{
    if !ctx.is_receive_enabled() {
        return Err(Error::receive_disabled());
    }

    let full_denom_path = data.denomination.as_str();
    let receiver = FromStr::from_str(data.receiver.as_str())?;

    if ctx.is_blocked_account(&receiver) {
        return Err(Error::receive_disabled());
    }
    match get_source_chain(
        &packet.destination_port,
        &packet.destination_channel,
        full_denom_path,
    ) {
        SourceChain::Receiver => {
            // Remove prefix added by sender chain
            let voucher_prefix =
                Denom::get_denom_prefix(&packet.source_port, &packet.source_channel);
            let unprefixed_denom = full_denom_path.to_string().split_off(voucher_prefix.len());

            let token = Coin {
                denom: unprefixed_denom.into(),
                amount: data.amount,
            };

            let escrow_address = ctx.get_channel_escrow_address(
                &packet.destination_port,
                &packet.destination_channel,
            )?;
            // Send tokens from escrow to receiver
            ctx.send_coins(&escrow_address, &receiver, &token)?;

            return Ok(ICS20Acknowledgement::Success);
        }
        _ => {}
    }

    let mut denom = Denom::get_denom_prefix(&packet.destination_port, &packet.destination_channel);
    // NOTE: source_prefix contains the trailing "/"
    denom.push_str(full_denom_path);

    let token = Coin {
        denom: denom.into(),
        amount: data.amount,
    };

    let module_acc = ctx.get_module_account();
    // Mint vouchers to module
    ctx.mint_coins(&token)?;
    // Send vouchers to receiver
    ctx.send_coins(&module_acc, &receiver, &token)?;
    Ok(ICS20Acknowledgement::Success)
}
