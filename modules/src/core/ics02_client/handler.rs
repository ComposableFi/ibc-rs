//! This module implements the processing logic for ICS2 (client abstractions and functions) msgs.

use crate::clients::GlobalDefs;

use crate::core::ics02_client::client_type::ClientTypes;
use crate::core::ics02_client::error::Error;
use crate::core::ics02_client::msgs::ClientMsg;
use crate::core::ics26_routing::context::ReaderContext;
use crate::handler::HandlerOutput;
use core::fmt::Debug;

pub mod create_client;
pub mod update_client;
pub mod upgrade_client;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientResult<C>
where
    C: ClientTypes + Clone + Debug + PartialEq + Eq,
{
    Create(create_client::Result<C>),
    Update(update_client::Result<C>),
    Upgrade(upgrade_client::Result<C>),
}

/// General entry point for processing any message related to ICS2 (client functions) protocols.
pub fn dispatch<Ctx, G: GlobalDefs>(
    ctx: &Ctx,
    msg: ClientMsg<<Ctx as ReaderContext>::ClientTypes>,
) -> Result<HandlerOutput<ClientResult<<Ctx as ReaderContext>::ClientTypes>>, Error>
where
    Ctx: ReaderContext<ClientTypes = G::ClientTypes>,
{
    match msg {
        ClientMsg::CreateClient(msg) => create_client::process::<G, _>(ctx, msg),
        ClientMsg::UpdateClient(msg) => update_client::process::<G, _>(ctx, msg),
        ClientMsg::UpgradeClient(msg) => upgrade_client::process::<G, _>(ctx, msg),
        _ => {
            unimplemented!()
        }
    }
}
