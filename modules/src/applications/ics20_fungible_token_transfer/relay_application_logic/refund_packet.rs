use crate::applications::ics20_fungible_token_transfer::context::Ics20Context;
use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::primitives::{
    Coin, FungibleTokenPacketData,
};
use crate::applications::ics20_fungible_token_transfer::utils::{get_source_chain, SourceChain};
use crate::core::ics04_channel::packet::Packet;
use alloc::str::FromStr;
use alloc::string::ToString;

/// Implements logic for refunding a sender on packet timeout or acknowledgement error
pub fn refund_packet_token<Ctx>(
    ctx: &mut Ctx,
    packet: &Packet,
    data: &FungibleTokenPacketData,
) -> Result<(), Error>
where
    Ctx: Ics20Context,
{
    let full_denom_path = data.denomination.as_str();

    let token = Coin {
        denom: full_denom_path.to_string().into(),
        amount: data.amount,
    };
    let sender = FromStr::from_str(data.sender.as_str())?;
    match get_source_chain(&packet.source_port, &packet.source_channel, full_denom_path) {
        SourceChain::Sender => {
            let escrow_address =
                ctx.get_channel_escrow_address(&packet.source_port, &packet.source_channel)?;

            ctx.send_coins(&escrow_address, &sender, &token)?;
            return Ok(());
        }
        _ => {}
    }

    // Mint vouchers back to sender
    ctx.mint_coins(&token)?;
    // Send back to sender
    let module_acc = ctx.get_module_account();
    ctx.send_coins(&module_acc, &sender, &token)?;
    Ok(())
}
