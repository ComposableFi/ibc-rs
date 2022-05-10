//! This module implements the processing logic for ICS2 (client abstractions and functions) msgs.

use crate::clients::ics11_beefy::client_def::BeefyLCStore;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::msgs::ClientMsg;
use crate::handler::HandlerOutput;

pub mod create_client;
pub mod update_client;
pub mod upgrade_client;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientResult {
    Create(create_client::Result),
    Update(update_client::Result),
    Upgrade(upgrade_client::Result),
}

/// General entry point for processing any message related to ICS2 (client functions) protocols.
pub fn dispatch<Ctx, Beefy>(ctx: &Ctx, msg: ClientMsg) -> Result<HandlerOutput<ClientResult>, Error>
where
    Ctx: ClientReader,
    Beefy: BeefyLCStore,
{
    match msg {
        ClientMsg::CreateClient(msg) => create_client::process(ctx, msg),
        ClientMsg::UpdateClient(msg) => update_client::process::<Beefy>(ctx, msg),
        ClientMsg::UpgradeClient(msg) => upgrade_client::process::<Beefy>(ctx, msg),
        _ => {
            unimplemented!()
        }
    }
}
