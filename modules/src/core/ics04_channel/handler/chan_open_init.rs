//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use crate::{
	core::{
		ics04_channel::{
			channel::{ChannelEnd, State},
			error::Error,
			events::Attributes,
			handler::{ChannelIdState, ChannelResult},
			msgs::chan_open_init::MsgChannelOpenInit,
		},
		ics24_host::identifier::ChannelId,
		ics26_routing::context::ReaderContext,
	},
	events::IbcEvent,
	handler::{HandlerOutput, HandlerResult},
	prelude::*,
};

pub(crate) fn process<Ctx: ReaderContext>(
	ctx: &Ctx,
	msg: &MsgChannelOpenInit,
) -> HandlerResult<ChannelResult, Error> {
	let mut output = HandlerOutput::builder();

	if msg.channel.connection_hops().len() != 1 {
		return Err(Error::invalid_connection_hops_length(1, msg.channel.connection_hops().len()))
	}

	// An IBC connection running on the local (host) chain should exist.
	let conn = ctx
		.connection_end(&msg.channel.connection_hops()[0])
		.map_err(Error::ics03_connection)?;
	let get_versions = conn.versions();
	let version = match get_versions {
		[version] => version,
		_ => return Err(Error::invalid_version_length_connection()),
	};

	let channel_feature = msg.channel.ordering().to_string();
	if !version.is_supported_feature(channel_feature) {
		return Err(Error::channel_feature_not_suported_by_connection())
	}

	// Channel identifier construction.
	let id_counter = ctx.channel_counter()?;
	let chan_id = ChannelId::new(id_counter);

	output.log(format!("success: generated new channel identifier: {}", chan_id));

	let new_channel_end = ChannelEnd::new(
		State::Init,
		*msg.channel.ordering(),
		msg.channel.counterparty().clone(),
		msg.channel.connection_hops().clone(),
		msg.channel.version().clone(),
	);

	output.log("success: no channel found");

	let event_attributes = Attributes {
		channel_id: Some(chan_id),
		height: ctx.host_height(),
		port_id: msg.port_id.clone(),
		connection_id: new_channel_end.connection_hops[0].clone(),
		counterparty_port_id: new_channel_end.counterparty().port_id.clone(),
		counterparty_channel_id: new_channel_end.counterparty().channel_id.clone(),
	};

	let result = ChannelResult {
		port_id: msg.port_id.clone(),
		channel_id: chan_id,
		channel_end: new_channel_end,
		channel_id_state: ChannelIdState::Generated,
	};

	output.emit(IbcEvent::OpenInitChannel(
		event_attributes.try_into().map_err(|_| Error::missing_channel_id())?,
	));

	Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
	use crate::prelude::*;

	use test_log::test;

	use crate::{
		core::{
			ics02_client::context::ClientReader,
			ics03_connection::{
				connection::{ConnectionEnd, State as ConnectionState},
				msgs::conn_open_init::{
					test_util::get_dummy_raw_msg_conn_open_init, MsgConnectionOpenInit,
				},
				version::get_compatible_versions,
			},
			ics04_channel::{
				channel::State,
				handler::channel_dispatch,
				msgs::{
					chan_open_init::{
						test_util::get_dummy_raw_msg_chan_open_init, MsgChannelOpenInit,
					},
					ChannelMsg,
				},
			},
			ics24_host::identifier::ConnectionId,
		},
		events::IbcEvent,
		mock::context::{MockClientTypes, MockContext},
	};

	#[test]
	fn chan_open_init_msg_processing() {
		struct Test {
			name: String,
			ctx: MockContext<MockClientTypes>,
			msg: ChannelMsg,
			want_pass: bool,
		}

		let msg_chan_init =
			MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init()).unwrap();

		let context = MockContext::default();

		let msg_conn_init =
			MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();

		let init_conn_end = ConnectionEnd::new(
			ConnectionState::Init,
			msg_conn_init.client_id.clone(),
			msg_conn_init.counterparty.clone(),
			get_compatible_versions(),
			msg_conn_init.delay_period,
		);

		let cid = ConnectionId::default();

		let tests: Vec<Test> = vec![
			Test {
				name: "Processing fails because no connection exists in the context".to_string(),
				ctx: context.clone(),
				msg: ChannelMsg::ChannelOpenInit(msg_chan_init.clone()),
				want_pass: false,
			},
			Test {
				name: "Good parameters".to_string(),
				ctx: context.with_connection(cid, init_conn_end),
				msg: ChannelMsg::ChannelOpenInit(msg_chan_init),
				want_pass: true,
			},
		]
		.into_iter()
		.collect();

		for test in tests {
			let res = channel_dispatch(&test.ctx, &test.msg);
			// Additionally check the events and the output objects in the result.
			match res {
				Ok((proto_output, res)) => {
					assert!(
                        test.want_pass,
                        "chan_open_init: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg,
                        test.ctx.clone()
                    );

					let proto_output = proto_output.with_result(());
					assert!(!proto_output.events.is_empty()); // Some events must exist.

					// The object in the output is a ChannelEnd, should have init state.
					assert_eq!(res.channel_end.state().clone(), State::Init);
					let msg_init = test.msg;

					if let ChannelMsg::ChannelOpenInit(msg_init) = msg_init {
						assert_eq!(res.port_id.clone(), msg_init.port_id.clone());
					}

					for e in proto_output.events.iter() {
						assert!(matches!(e, &IbcEvent::OpenInitChannel(_)));
						assert_eq!(e.height(), test.ctx.host_height());
					}
				},
				Err(e) => {
					assert!(
						!test.want_pass,
						"chan_open_init: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
						test.name,
						test.msg,
						test.ctx.clone(),
						e,
					);
				},
			}
		}
	}
}
